use crate::net::message::header::Header;
use crate::net::message::{MessageDirection, MessageId, MessageKind, HEADER_SIZE};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::{Display, Formatter};

pub const MAX_MESSAGE_SIZE: usize = 4096;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Message {
    header: Header,
    data: Bytes,
}

impl Message {
    pub fn new(dir: MessageDirection, kind: MessageKind, op: usize, data: Bytes) -> Self {
        Self {
            header: Header::new(
                MessageId::new()
                    .with_kind(kind)
                    .with_direction(dir)
                    .with_operation(op),
                data.len() as u16,
            ),
            data,
        }
    }

    pub fn is_encrypted(&self) -> bool {
        self.header.data_size() & 0x8000 != 0
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn reader(self) -> Bytes {
        self.data
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let emoji = if self.is_encrypted() { "🔐" } else { "🔓" };
        write!(f, "{} {}: {:X}", emoji, self.header, self.data)
    }
}

impl From<(Header, Bytes)> for Message {
    fn from(value: (Header, Bytes)) -> Self {
        Self {
            header: value.0,
            data: value.1,
        }
    }
}

impl From<Bytes> for Message {
    fn from(mut value: Bytes) -> Self {
        Self {
            header: Header::from(value.copy_to_bytes(HEADER_SIZE)),
            data: value,
        }
    }
}

impl From<&[u8]> for Message {
    fn from(data: &[u8]) -> Self {
        Message {
            header: Header::from(&data[0..HEADER_SIZE]),
            data: Bytes::copy_from_slice(&data[6..]),
        }
    }
}

impl<'a> Into<Bytes> for Message {
    fn into(self) -> Bytes {
        let mut mem = BytesMut::new();

        mem.put::<Bytes>(self.header.into());
        mem.put::<Bytes>(self.data);

        mem.freeze()
    }
}
