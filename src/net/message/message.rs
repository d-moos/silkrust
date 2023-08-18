use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::net::message::header::Header;

const MAX_HEADER_SIZE: usize = 6;

pub struct Message {
    header: Header,
    data: Bytes
}

impl From<Bytes> for Message {
    fn from(mut value: Bytes) -> Self {
        Self {
            header: Header::from(value.copy_to_bytes(MAX_HEADER_SIZE)),
            data: value
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