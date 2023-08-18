use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::net::message::id::MessageId;

/// Message Header
///
/// The header of a network message which is always exactly 6 bytes
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