use crate::security::blowfish_compat::{Block, BlockEncrypt, BlowfishCompat, NewBlockCipher};

pub type Signature = [u8; 8];
pub type Key = [u8; 8];

#[derive(Default, Copy, Clone)]
pub struct NotSet;

#[derive(Default)]
pub struct Set(u32);

#[derive(Default, Copy, Clone)]
pub struct Exchange<TRemote> {
    initial: Key,

    generator: u32,
    prime: u32,
    private: u32,

    remote: TRemote,
}

impl<TRemote> Exchange<TRemote> {
    pub fn set_initial(self, initial: Key) -> Self {
        Self { initial, ..self }
    }

    pub fn get_initial(&self) -> Key {
        self.initial
    }

    pub fn set_generator(self, generator: u32) -> Self {
        Self { generator, ..self }
    }

    pub fn get_generator(&self) -> u32 {
        self.generator
    }

    pub fn set_prime(self, prime: u32) -> Self {
        Self { prime, ..self }
    }

    pub fn get_prime(&self) -> u32 {
        self.prime
    }

    pub fn set_private(self, private: u32) -> Self {
        Self { private, ..self }
    }

    pub fn get_local(&self) -> u32 {
        g_pow_x_mod_p(self.generator, self.private, self.prime.into())
    }
}

impl Exchange<NotSet> {
    pub fn remote(self, remote: u32) -> Exchange<Set> {
        Exchange {
            remote: Set(remote),
            generator: self.generator,
            initial: self.initial,
            prime: self.prime,
            private: self.private,
        }
    }
}

impl Exchange<Set> {
    pub fn remote(&self) -> u32 {
        self.remote.0
    }

    pub fn shared(&self) -> u32 {
        g_pow_x_mod_p(self.remote.0, self.private, self.prime.into())
    }
}

pub struct ChallengeMismatch;

pub struct Initiator;
pub struct Responder;

impl Challenge for Initiator {
    fn key(context: &Exchange<Set>) -> Key {
        calculate_key(context.shared(), context.get_local(), context.remote())
    }
}

impl Challenge for Responder {
    fn key(context: &Exchange<Set>) -> Key {
        calculate_key(context.shared(), context.remote(), context.get_local())
    }
}

pub trait Challenge {
    fn key(context: &Exchange<Set>) -> Key;

    fn create(context: &Exchange<Set>) -> Signature {
        let key = Self::key(context);
        let mut signature =
            calculate_signature(context.shared(), context.get_local(), context.remote());

        // encrypt signature
        let blowfish = BlowfishCompat::new_from_slice(&key).expect("could not initialize blowfish");
        blowfish.encrypt_block(Block::from_mut_slice(&mut signature));

        signature
    }

    fn verify(context: &Exchange<Set>, challenge: Signature) -> Result<(), ChallengeMismatch> {
        let key = Self::key(context);
        let mut signature =
            calculate_signature(context.shared(), context.remote(), context.get_local());

        // encrypt signature
        let blowfish = BlowfishCompat::new_from_slice(&key).expect("could not initialize blowfish");
        blowfish.encrypt_block(Block::from_mut_slice(&mut signature));

        if signature == challenge {
            Ok(())
        } else {
            Err(ChallengeMismatch)
        }
    }

    fn finalize(context: &Exchange<Set>) -> Key {
        let shared = context.shared();
        let mut initial = context.initial.clone();
        transform_value(initial.as_mut(), shared, 3);
        context.initial
    }
}

fn calculate_signature(shared_secret: u32, secret1: u32, secret2: u32) -> Signature {
    let mut signature: [u8; 8] = [0; 8];
    signature[0..4].copy_from_slice(secret1.to_le_bytes().as_slice());
    signature[4..8].copy_from_slice(secret2.to_le_bytes().as_slice());

    transform_value(signature.as_mut(), shared_secret, (secret1 & 7) as u8);

    signature as Signature
}

fn calculate_key(shared_secret: u32, secret1: u32, secret2: u32) -> Key {
    let mut key: [u8; 8] = [0; 8];
    key[0..4].copy_from_slice(secret1.to_le_bytes().as_slice());
    key[4..8].copy_from_slice(secret2.to_le_bytes().as_slice());

    transform_value(key.as_mut(), shared_secret, (shared_secret & 3) as u8);

    key as Key
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
    use crate::security::{Challenge, Exchange, Initiator, Key, NotSet, Responder};

    #[test]
    fn roundtrip() {
        let initiator_private: u32 = rand::random();
        let responder_private: u32 = rand::random();
        let (g, prime, initial): (u32, u32, Key) = (
            rand::random::<u32>() & 0x7FFFFFFF,
            rand::random::<u32>() & 0x7FFFFFFF,
            rand::random()
        );

        let initiator = Exchange::<NotSet>::default()
            .set_initial(initial)
            .set_generator(g)
            .set_prime(prime)
            .set_private(initiator_private);

        let responder = Exchange::<NotSet>::default()
            .set_initial(initial)
            .set_generator(g)
            .set_prime(prime)
            .set_private(initiator_private)
            .remote(initiator.get_local());

        let initiator = initiator.remote(responder.get_local());

        let responder_challenge = <Responder as Challenge>::create(&responder);
        assert!(<Initiator as Challenge>::verify(&initiator, responder_challenge).is_ok());

        let initiator_challenge = <Initiator as Challenge>::create(&initiator);
        assert!(<Responder as Challenge>::verify(&responder, initiator_challenge).is_ok());
    }
}
