/// A struct keeping track of the DH key exchange state
///
/// `SecretContext` keeps track of all data that is necessary for a [Diffie Hellman key exchange](https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange)
/// - `Initialized`: Only the local side is known and the remote public has not been shared yet
/// - `Finalized`: All relevant data is known and the shared_secret has been calculated
pub enum SecretContext {
    Initialized(PartialContext),
    Finalized(CompleteContext),
}

pub enum CalculationError {
    InvalidStage,
}

impl SecretContext {
    /// Initializes a new SecretContext and puts it into the `Initialized` stage
    pub fn initialize(generator: u32, prime: u32, private: u32) -> Self {
        SecretContext::Initialized(PartialContext {
            generator,
            prime,
            private,
            local_public: g_pow_x_mod_p(generator, prime, prime.into()),
        })
    }

    /// Finalizes a previously `Initialized` SecretContext and puts it into the `Finalized` stage.
    /// if the SecretContext is already in the `Finalized` stage, the origin context is returned.
    pub fn finalize(self, remote_public: u32) -> Self {
        match self {
            SecretContext::Initialized(i) => SecretContext::Finalized(CompleteContext {
                local_public: i.local_public,
                remote_public,
                shared_secret: g_pow_x_mod_p(remote_public, i.private, i.prime.into()),
            }),
            SecretContext::Finalized(_) => self,
        }
    }

    /// Calculates the local signature from a `Finalized` SecretContext.
    ///
    /// # Errors
    ///
    /// If the function is called on an `Initialized` SecretContext,
    /// a `CalculationError::InvalidStage` error is returned.
    pub fn local_signature(&self) -> Result<Signature, CalculationError> {
        let context = self.ensure_is_finalized()?;
        Ok(calculate_signature(
            context.shared_secret,
            context.local_public,
            context.remote_public,
        ))
    }

    /// Calculates the remote signature from a `Finalized` SecretContext.
    ///
    /// # Errors
    ///
    /// If the function is called on an `Initialized` SecretContext,
    /// a `CalculationError::InvalidStage` error is returned.
    pub fn remote_signature(&self) -> Result<Signature, CalculationError> {
        let context = self.ensure_is_finalized()?;
        Ok(calculate_signature(
            context.shared_secret,
            context.remote_public,
            context.local_public,
        ))
    }

    /// Calculates a blowfish key from a `Finalized` SecretContext
    ///
    /// # Errors
    ///
    /// If the function is called on an `Initialized` SecretContext,
    /// a `CalculationError::InvalidStage` error is returned.
    pub fn blowfish_key(&self) -> Result<BlowfishKey, CalculationError> {
        let context = self.ensure_is_finalized()?;
        Ok(calculate_key(
            context.shared_secret,
            context.remote_public,
            context.local_public,
        ))
    }
    fn ensure_is_finalized(&self) -> Result<&CompleteContext, CalculationError> {
        match self {
            SecretContext::Initialized(_) => Err(CalculationError::InvalidStage),
            SecretContext::Finalized(c) => Ok(c),
        }
    }
}

struct PartialContext {
    generator: u32,
    prime: u32,
    private: u32,
    local_public: u32,
}

struct CompleteContext {
    local_public: u32,
    remote_public: u32,
    shared_secret: u32,
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
