use crate::net::message::header::Header;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::net::message::HEADER_SIZE;

pub const MAX_MESSAGE_SIZE: usize = 4096;

pub struct Message {
    header: Header,
    data: Bytes,
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
            data: Bytes::copy_from_slice(&data[6..])
        }
    }
}


impl Into<Bytes> for Message {
    fn into(self) -> Bytes {
        let mut mem = BytesMut::new();

        mem.put::<Bytes>(self.header.into());
        mem.put::<Bytes>(self.data);

        mem.freeze()
    }
}