use bitfield_struct::bitfield;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::security::{Key, Signature};

#[bitfield(u8)]
pub struct HandshakeOptions {
    pub disabled: bool,
    pub encryption: bool,
    pub error_detection: bool,
    pub exchange: bool,
    pub challenge: bool,
    #[bits(3)]
    _padding: u8,
}

pub struct ErrorDetectionSeed {
    pub sequence: u32,
    pub checksum: u32,
}

impl From<Bytes> for ErrorDetectionSeed {
    fn from(mut value: Bytes) -> Self {
        ErrorDetectionSeed {
            sequence: value.get_u32_le(),
            checksum: value.get_u32_le(),
        }
    }
}

impl Into<Bytes> for ErrorDetectionSeed {
    fn into(self) -> Bytes {
        let mut mem = BytesMut::new();
        mem.put_u32_le(self.sequence);
        mem.put_u32_le(self.checksum);
        mem.freeze()
    }
}

impl ErrorDetectionSeed {
    pub fn new(sequence: u32, checksum: u32) -> Self {
        Self {
            sequence,
            checksum
        }
    }
}

pub struct ExchangeSetup {
    pub initial_key: Key,
    pub generator: u32,
    pub prime: u32,
    pub public: u32,
}

impl From<Bytes> for ExchangeSetup {
    fn from(mut value: Bytes) -> Self {
        let mut initial_key = Key::default();
        value.copy_to_slice(initial_key.as_mut_slice());

        Self {
            initial_key,
            generator: value.get_u32_le(),
            prime: value.get_u32_le(),
            public: value.get_u32_le(),
        }
    }
}

impl Into<Bytes> for ExchangeSetup {
    fn into(self) -> Bytes {
        let mut mem = BytesMut::new();
        mem.put_slice(self.initial_key.as_slice());
        mem.put_u32_le(self.generator);
        mem.put_u32_le(self.prime);
        mem.put_u32_le(self.public);
        mem.freeze()
    }
}

impl ExchangeSetup {
    pub fn new(initial_key: Key, generator: u32, prime: u32, public: u32) -> Self {
        Self {
            initial_key,
            generator,
            prime,
            public
        }
    }
}

#[derive(Debug)]
pub struct ExchangeResponse {
    pub public: u32,
    pub signature: Signature,
}

impl Into<Bytes> for ExchangeResponse {
    fn into(self) -> Bytes {
        let mut mem = BytesMut::new();
        mem.put_u32_le(self.public);
        mem.put_slice(self.signature.as_slice());
        mem.freeze()
    }
}

impl From<Bytes> for ExchangeResponse {
    fn from(mut value: Bytes) -> Self {
        let public = value.get_u32_le();
        let mut signature = Signature::default();
        value.copy_to_slice(signature.as_mut_slice());

        Self {
            public,
            signature
        }
    }
}

impl ExchangeResponse {
    pub fn new(public: u32, signature: Signature) -> Self {
        Self {
            public,
            signature
        }
    }
}