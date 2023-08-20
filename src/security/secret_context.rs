pub struct SecretContext {
    initial_key: BlowfishKey,
    generator: u32,
    prime: u32,
    private: u32,
    remote_public: Option<u32>,
}

#[derive(Debug)]
pub struct RemotePublicNotSet;

impl SecretContext {
    pub fn new(
        initial_key: BlowfishKey,
        generator: u32,
        prime: u32,
        private: u32,
        remote_public: Option<u32>,
    ) -> Self {
        Self {
            initial_key,
            generator,
            prime,
            private,
            remote_public,
        }
    }

    pub fn local_public(&self) -> u32 {
        g_pow_x_mod_p(self.generator, self.private, self.prime.into())
    }

    pub fn set_remote_public(&mut self, remote_public: u32) {
        self.remote_public = Some(remote_public)
    }

    pub fn remote_public(&self) -> Result<u32, RemotePublicNotSet> {
        self.remote_public.ok_or(RemotePublicNotSet)
    }

    pub fn shared_secret(&self) -> Result<u32, RemotePublicNotSet> {
        self.remote_public
            .map(|r| g_pow_x_mod_p(r, self.private, self.prime.into()))
            .ok_or(RemotePublicNotSet)
    }

    pub fn local_signature(&self) -> Result<Signature, RemotePublicNotSet> {
        self.remote_public
            .map(|r| calculate_signature(self.shared_secret().unwrap(), self.local_public(), r))
            .ok_or(RemotePublicNotSet)
    }

    pub fn remote_signature(&self) -> Result<Signature, RemotePublicNotSet> {
        self.remote_public
            .map(|r| calculate_signature(self.shared_secret().unwrap(), r, self.local_public()))
            .ok_or(RemotePublicNotSet)
    }

    pub fn intermediary_key(&self) -> Result<BlowfishKey, RemotePublicNotSet> {
        self.remote_public
            .map(|r| calculate_key(self.shared_secret().unwrap(), r, self.local_public()))
            .ok_or(RemotePublicNotSet)
    }

    pub fn final_key(&self) -> Result<BlowfishKey, RemotePublicNotSet> {
        let mut key = BlowfishKey::default();
        key.copy_from_slice(self.initial_key.as_slice());

        self.shared_secret().map(|secret| {
            transform_value(key.as_mut(), secret, 3);
            key
        })
    }
}

/// A signature that is used to verify a successful secret exchange
pub type Signature = [u8; 8];

/// A key that is used to initialize the [blowfish_compat::BlowfishCompat] cipher
pub type BlowfishKey = [u8; 8];

fn calculate_signature(shared_secret: u32, secret1: u32, secret2: u32) -> Signature {
    let mut signature: [u8; 8] = [0; 8];
    signature[0..4].copy_from_slice(secret1.to_le_bytes().as_slice());
    signature[4..8].copy_from_slice(secret2.to_le_bytes().as_slice());

    transform_value(signature.as_mut(), shared_secret, (secret1 & 7) as u8);

    signature as Signature
}

fn calculate_key(shared_secret: u32, secret1: u32, secret2: u32) -> BlowfishKey {
    let mut key: [u8; 8] = [0; 8];
    key[0..4].copy_from_slice(secret1.to_le_bytes().as_slice());
    key[4..8].copy_from_slice(secret2.to_le_bytes().as_slice());

    transform_value(key.as_mut(), shared_secret, (shared_secret & 3) as u8);

    key as BlowfishKey
}

fn g_pow_x_mod_p(g: u32, mut x: u32, p: i64) -> u32 {
    let mut current: i64 = 1;
    let mut mult: i64 = g as i64;

    if x == 0 {
        return 1;
    }

    while x != 0 {
        if (x & 1) > 0 {
            current = (mult * current) % p;
        }
        x >>= 1;
        mult = (mult * mult) % p;
    }
    current as u32
}

fn transform_value(value: &mut [u8], key: u32, key_byte: u8) {
    value[0] ^= value[0]
        .wrapping_add((key >> 00 & 0xFF) as u8)
        .wrapping_add(key_byte);
    value[1] ^= value[1]
        .wrapping_add((key >> 08 & 0xFF) as u8)
        .wrapping_add(key_byte);
    value[2] ^= value[2]
        .wrapping_add((key >> 16 & 0xFF) as u8)
        .wrapping_add(key_byte);
    value[3] ^= value[3]
        .wrapping_add((key >> 24 & 0xFF) as u8)
        .wrapping_add(key_byte);

    value[4] ^= value[4]
        .wrapping_add((key >> 00 & 0xFF) as u8)
        .wrapping_add(key_byte);
    value[5] ^= value[5]
        .wrapping_add((key >> 08 & 0xFF) as u8)
        .wrapping_add(key_byte);
    value[6] ^= value[6]
        .wrapping_add((key >> 16 & 0xFF) as u8)
        .wrapping_add(key_byte);
    value[7] ^= value[7]
        .wrapping_add((key >> 24 & 0xFF) as u8)
        .wrapping_add(key_byte);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_with_known_result() {
        assert_eq!(g_pow_x_mod_p(10, 20, 30), 10);
    }

    #[test]
    fn compare_transform_with_known_result() {
        let expected: [u8; 8] = [58, 49, 1, 1, 58, 49, 1, 1];
        let mut buf: [u8; 8] = [0; 8];
        let key = 12345;
        transform_value(&mut buf, key, (key & 7) as u8);

        assert_eq!(buf, expected);
    }
}
