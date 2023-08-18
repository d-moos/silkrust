use std::fmt::{Display, Formatter};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::net::message::id::MessageId;

pub const HEADER_SIZE: usize = 6;
const HEADER_ENC_MASK: u16 = 0x8000;

/// Message Header
///
/// The header of a network message which is always exactly 6 bytes
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Header {
    /// message data size
    ///
    /// An MSB of 1 means that the message is encrypted.
    size: u16,

    /// message id
    id: MessageId,

    /// sequential check, ensures a message is not being replayed
    sequence: u8,

    /// cyclic redundancy check
    checksum: u8,
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.id
        )
    }
}

impl Header {
    pub fn new(id: MessageId, size: u16) -> Self {
        Self {
            id,
            size,
            checksum: 0,
            sequence: 0
        }
    }

    pub fn message_size(&self) -> u16 {
        self.data_size() + HEADER_SIZE as u16
    }

    pub fn data_size(&self) -> u16 {
        self.size & !HEADER_ENC_MASK
    }
}

impl From<Bytes> for Header {
    fn from(mut value: Bytes) -> Self {
        Self {
            size: value.get_u16_le(),
            id: MessageId::from(value.get_u16_le()),
            sequence: value.get_u8(),
            checksum: value.get_u8()
        }
    }
}

impl From<&[u8]> for Header {
    fn from(buffer: &[u8]) -> Self {
        Header {
            size: u16::from_le_bytes(buffer[0..2].try_into().unwrap()),
            id: MessageId::from(u16::from_le_bytes(buffer[2..4].try_into().unwrap())),
            checksum: buffer[5],
            sequence: buffer[4],
        }
    }
}

impl Into<Bytes> for Header {
    fn into(self) -> Bytes {
        let mut mem = BytesMut::new();
        mem.put_u16_le(self.size);
        mem.put_u16_le(self.id.into());
        mem.put_u8(self.sequence);
        mem.put_u8(self.checksum);

        mem.freeze()
    }
}

// TODO: write a test with a known byte array to check if conversions are correct.