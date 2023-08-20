use crate::net::message::MessageDirection::{Ack, Req};
use crate::net::message::{Header, Message, MessageDirection, MessageId, MessageKind};
use crate::net::{MessageTable, NetClient, Process};
use crate::security::{BlowfishKey, SecretContext, SecurityBuilder, Signature};
use bitfield_struct::bitfield;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[bitfield(u8)]
struct HandshakeOptions {
    disabled: bool,
    encryption: bool,
    error_detection: bool,
    exchange: bool,
    challenge: bool,
    #[bits(3)]
    _padding: u8,
}

struct ErrorDetectionSeed {
    sequence: u32,
    checksum: u32,
}

impl From<Bytes> for ErrorDetectionSeed {
    fn from(mut value: Bytes) -> Self {
        ErrorDetectionSeed {
            sequence: value.get_u32_le(),
            checksum: value.get_u32_le(),
        }
    }
}

struct ExchangeSetup {
    initial_key: BlowfishKey,
    generator: u32,
    prime: u32,
    public: u32,
}

impl From<Bytes> for ExchangeSetup {
    fn from(mut value: Bytes) -> Self {
        let mut initial_key = BlowfishKey::default();
        value.copy_to_slice(initial_key.as_mut_slice());

        ExchangeSetup {
            initial_key,
            generator: value.get_u32_le(),
            prime: value.get_u32_le(),
            public: value.get_u32_le(),
        }
    }
}

struct ExchangeResponse {
    local_public: u32,
    signature: Signature,
}

impl Into<Bytes> for ExchangeResponse {
    fn into(self) -> Bytes {
        let mut buf = BytesMut::new();

        buf.put_u32_le(self.local_public);
        buf.put_slice(self.signature.as_slice());

        buf.freeze()
    }
}

pub struct HandshakeReqProcessor {
    secret_context: Option<SecretContext>,
}

impl Default for HandshakeReqProcessor {
    fn default() -> Self {
        Self {
            secret_context: None,
        }
    }
}

impl Process for HandshakeReqProcessor {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        let mut reader = m.reader();

        let options = HandshakeOptions::from(reader.get_u8());
        let mut security_builder = SecurityBuilder::default();

        if options.encryption() {
            let mut key_buffer = BlowfishKey::default();
            reader.copy_to_slice(key_buffer.as_mut_slice());
            security_builder = security_builder.blowfish(key_buffer);
        }

        if options.error_detection() {
            let error_detection = ErrorDetectionSeed::from(reader.copy_to_bytes(8));
            security_builder = security_builder
                .error_detection((error_detection.sequence, error_detection.checksum));
        }

        let (response, security) = if options.exchange() {
            let setup = ExchangeSetup::from(reader.copy_to_bytes(20));
            let secret_context = SecretContext::new(
                setup.initial_key,
                setup.generator,
                setup.prime,
                rand::random(),
                Some(setup.public),
            );

            security_builder = security_builder.blowfish(
                secret_context
                    .intermediary_key()
                    .expect("we can safely unwrap as the remote public should be set above"),
            );
            let security = security_builder.build();

            let mut signature = secret_context
                .local_signature()
                .expect("we can safely unwrap as the remote public should be set above");

            security.encrypt(&mut signature);

            let response = ExchangeResponse {
                signature,
                local_public: secret_context.local_public(),
            };

            self.secret_context = Some(secret_context);

            let mem: Bytes = response.into();
            (
                Message::new(
                    Header::new(
                        MessageId::new()
                            .with_operation(0)
                            .with_kind(MessageKind::NetEngine)
                            .with_direction(Req),
                        mem.len() as u16,
                    ),
                    mem,
                ),
                security,
            )
        } else {
            (
                Message::new(
                    Header::new(
                        MessageId::new()
                            .with_operation(0)
                            .with_kind(MessageKind::NetEngine)
                            .with_direction(Ack),
                        0,
                    ),
                    Bytes::new(),
                ),
                security_builder.build(),
            )
        };

        net_client.set_security(security);
        net_client.send(response);
    }
}
