use bitfield_struct::bitfield;
use blowfish_compat::{BlowfishCompat, NewBlockCipher};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use silkrust::net::message::MessageDirection::{Ack, Req};
use silkrust::net::message::MessageKind::NetEngine;
use silkrust::net::message::{Header, Message, MessageId, MessageKind};
use silkrust::net::{NetClient, Process};
use silkrust::security::{BlowfishKey, SecretContext, Security, SecurityBuilder, Signature};

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

        if options.challenge() {
            self.process_challenge(net_client, reader);
        } else {
            let mut security_builder = SecurityBuilder::default();
            security_builder = self.handle_encryption(options, &mut reader, security_builder);
            security_builder = self.handle_error_detection(options, &mut reader, security_builder);

            let (response, security) = if options.exchange() {
                self.handle_exchange(security_builder, &mut reader)
            } else {
                self.handle_no_exchange(security_builder)
            };

            net_client.set_security(security);
            net_client.send(response);
        }
    }
}

impl HandshakeReqProcessor {
    fn handle_encryption(
        &self,
        options: HandshakeOptions,
        reader: &mut Bytes,
        security_builder: SecurityBuilder,
    ) -> SecurityBuilder {
        if options.encryption() {
            let mut key_buffer = BlowfishKey::default();
            reader.copy_to_slice(key_buffer.as_mut_slice());
            return security_builder.blowfish(key_buffer);
        }
        security_builder
    }

    fn handle_error_detection(
        &self,
        options: HandshakeOptions,
        reader: &mut Bytes,
        security_builder: SecurityBuilder,
    ) -> SecurityBuilder {
        if options.error_detection() {
            let error_detection = ErrorDetectionSeed::from(reader.copy_to_bytes(8));
            return security_builder
                .error_detection((error_detection.sequence, error_detection.checksum));
        }

        security_builder
    }

    fn handle_exchange(
        &mut self,
        mut security_builder: SecurityBuilder,
        reader: &mut Bytes,
    ) -> (Message, Security) {
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
        (Message::new(Req, NetEngine, 0, mem), security)
    }

    fn handle_no_exchange(&self, mut security_builder: SecurityBuilder) -> (Message, Security) {
        (
            Message::new(Ack, NetEngine, 0, Bytes::new()),
            security_builder.build(),
        )
    }

    fn process_challenge(&mut self, net_client: &mut NetClient, mut reader: Bytes) {
        let secret_context = self.secret_context.as_ref().expect("asdf");
        let mut given_remote_signature = Signature::default();
        reader.copy_to_slice(&mut given_remote_signature);

        let mut calculated_remote_signature = secret_context.remote_signature().expect("asdf");
        if let Some(security) = net_client.security_mut() {
            security.encrypt(calculated_remote_signature.as_mut_slice());
        }
        // todo: handle gracefully and disconnect client
        assert_eq!(
            calculated_remote_signature, given_remote_signature,
            "remote signature missmatch"
        );

        let final_key = secret_context.final_key().expect("asdf");
        if let Some(security) = net_client.security_mut() {
            security.blowfish =
                Some(BlowfishCompat::new_from_slice(final_key.as_slice()).expect("asdf"));
        }

        net_client.send(Message::new(Ack, NetEngine, 0, Bytes::new()));
    }
}
