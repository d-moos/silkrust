use crate::net::message::Message;
use crate::security::blowfish_compat::{
    Block, BlockDecrypt, BlockEncrypt, BlowfishCompat, NewBlockCipher,
};
use crate::security::{Checksum, Key, Sequencer};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::{error, warn};

pub struct EncodingRequirements {
    outbound: bool,
    inbound: bool,
}

impl EncodingRequirements {
    pub fn new(inbound: bool, outbound: bool) -> Self {
        Self { inbound, outbound }
    }
}

struct Encoder {
    requirements: EncodingRequirements,
    sequencer: Sequencer,
    checksum: Checksum,
}

impl Encoder {
    fn new(inbound: bool, outbound: bool, sequencer_seed: u32, checksum_seed: u32) -> Self {
        Self {
            requirements: EncodingRequirements::new(inbound, outbound),
            sequencer: Sequencer::new(sequencer_seed),
            checksum: Checksum::new(checksum_seed),
        }
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self {
            checksum: Checksum::default(),
            sequencer: Sequencer::default(),
            requirements: EncodingRequirements::new(false, false),
        }
    }
}

pub struct Security {
    pub blowfish: Option<BlowfishCompat>,
    encoder: Encoder,
}

impl Default for Security {
    fn default() -> Self {
        Self {
            blowfish: None,
            encoder: Encoder::default(),
        }
    }
}

impl Security {
    pub fn encode(&mut self, mut message: Message) -> Message {
        if !self.encoder.requirements.outbound {
            return message;
        }

        let encoder = &mut self.encoder;
        message.header_mut().sequence = encoder.sequencer.next();
        message.header_mut().checksum = 0;

        let bytes: Bytes = message.into();
        let checksum = encoder.checksum.compute(bytes.as_ref(), bytes.len());

        let mut message = Message::from(bytes);

        message.header_mut().checksum = checksum;

        message
    }

    pub fn encrypt(&self, data: &mut [u8]) {
        if let Some(blowfish) = self.blowfish {
            blowfish.encrypt_block(Block::from_mut_slice(data));
        } else {
            warn!("encrypt called with uninitialized blowfish!");
        }
    }

    pub fn decrypt(&self, message: Message) -> Message {
        if message.is_encrypted() {
            if let Some(blowfish) = self.blowfish {
                let mut bytes: Bytes = message.into();
                let size = bytes.get_u16_le();
                let mut remaining = bytes.to_vec();
                let data = remaining.as_mut_slice();
                blowfish.decrypt_block(Block::from_mut_slice(data));

                let mut mem = BytesMut::new();
                mem.put_u16_le(size);
                mem.put_slice(data);

                mem.freeze().into()
            } else {
                error!("received encrypted message, but blowfish is not setup!");
                panic!();
            }
        } else {
            message
        }
    }
}

pub struct SecurityBuilder {
    key: Option<Key>,
    encoding_requirements: (/* inbound */ bool, /* outbound */ bool),
    error_detection: (/* sequencer */ u32, /* checksum */ u32),
}

impl Default for SecurityBuilder {
    fn default() -> Self {
        Self {
            key: None,
            error_detection: (0, 0),
            encoding_requirements: (false, false),
        }
    }
}

impl SecurityBuilder {
    pub fn blowfish(self, key: Key) -> Self {
        Self {
            error_detection: self.error_detection,
            encoding_requirements: self.encoding_requirements,
            key: Some(key),
        }
    }

    pub fn encoding_requirements(
        self,
        encoding_requirements: (/* inbound */ bool, /* outbound */ bool),
    ) -> Self {
        Self {
            encoding_requirements,
            error_detection: self.error_detection,
            key: self.key,
        }
    }

    pub fn error_detection(
        self,
        error_detection: (/* sequencer */ u32, /* checksum */ u32),
    ) -> Self {
        Self {
            encoding_requirements: self.encoding_requirements,
            error_detection,
            key: self.key,
        }
    }

    pub fn build(self) -> Security {
        Security {
            blowfish: self.key.map(|k| {
                BlowfishCompat::new_from_slice(k.as_slice())
                    .expect("blowfish initialization failed")
            }),
            encoder: Encoder::new(
                self.encoding_requirements.0,
                self.encoding_requirements.1,
                self.error_detection.0,
                self.error_detection.1,
            ),
        }
    }
}
