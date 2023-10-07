use crate::net::io::fragment::Fragment;
use bytes::{Buf, Bytes};
use std::string::FromUtf8Error;

pub trait BytesExtension {
    fn get_collection<T: Fragment>(&mut self) -> Vec<T>;
    fn get_string(&mut self) -> Result<String, FromUtf8Error>;
}

impl BytesExtension for Bytes {
    fn get_collection<T: Fragment>(&mut self) -> Vec<T> {
        let mut entities = vec![];
        while self.get_u8() == 1 {
            entities.push(<T as Fragment>::get(self));
        }

        entities
    }

    fn get_string(&mut self) -> Result<String, FromUtf8Error> {
        let mut buf = vec![0u8; self.get_u16_le() as usize];
        self.copy_to_slice(&mut buf);

        String::from_utf8(buf)
    }
}
