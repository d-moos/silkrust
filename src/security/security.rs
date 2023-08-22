use crate::security::{BlowfishKey, Checksum, Sequencer};
use blowfish_compat::{Block, BlockEncrypt, BlowfishCompat, NewBlockCipher};
use bytes::Bytes;
use log::warn;
use crate::net::message::Message;

struct Encoder {
    sequencer: Sequencer,
    checksum: Checksum,
}

pub struct Security {
    pub(crate) blowfish: Option<BlowfishCompat>,
    encoder: Option<Encoder>,
}

impl Security {
    pub fn encode(&mut self, mut message: Message) -> Message {
        if let Some(encoder) = &mut self.encoder {
            message.header_mut().sequence = encoder.sequencer.next();

            let bytes: Bytes = message.into();
            let checksum = encoder.checksum.compute(bytes.as_ref(), bytes.len());

            let mut message = Message::from(bytes);

            message.header_mut().checksum = checksum;

            message
        } else {
            message
        }
    }

    pub fn encrypt(&self, data: &mut [u8]) {
        if let Some(blowfish) = self.blowfish {
            blowfish.encrypt_block(Block::from_mut_slice(data));
        } else {
            warn!("encrypt called with uninitialized blowfish!");
        }
    }
}

pub struct SecurityBuilder {
    key: Option<BlowfishKey>,
    error_detection: Option<(u32, u32)>,
}

impl Default for SecurityBuilder {
    fn default() -> Self {
        Self {
            key: None,
            error_detection: None,
        }
    }
}

impl SecurityBuilder {
    pub fn blowfish(self, key: BlowfishKey) -> Self {
        Self {
            error_detection: self.error_detection,
            key: Some(key),
        }
    }

    pub fn error_detection(self, error_detection: (u32, u32)) -> Self {
        Self {
            error_detection: Some(error_detection),
            key: self.key,
        }
    }

    pub fn build(self) -> Security {
        Security {
            blowfish: self.key.map(|k| {
                BlowfishCompat::new_from_slice(k.as_slice())
                    .expect("blowfish initialization failed")
            }),
            encoder: self.error_detection.map(|t| Encoder {
                sequencer: Sequencer::new(t.0),
                checksum: Checksum::new(t.1),
            }),
        }
    }
}
