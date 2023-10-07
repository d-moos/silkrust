//! (taken from: https://git.eternalwings.de/tim/blowfish-compat.rs)
//! Wrapper under the blowfish algorithm
//! Implements compatibility with
//! - https://stackoverflow.com/a/11423057/829264
//! - https://github.com/winlibs/libmcrypt/blob/master/modules/algorithms/blowfish-compat.c
//!
//! Supports `#![no_std]`.
//! Optionally supports `bswap` crate as feature.

extern crate blowfish;
pub use blowfish::*;

use cipher::{
    consts::{U56, U8},
    errors::InvalidLength,
    generic_array::GenericArray,
    BlockCipher,
};

pub use cipher::{BlockDecrypt, BlockEncrypt, NewBlockCipher};

#[cfg(feature = "bswap")]
extern crate bswap;

/// Size (length) of word
pub(crate) const WORD: usize = 4;
/// Size (length) of chunk/block
pub const BLOCK_SIZE: usize = 8;

/**
Takes data and reverses byte order inplace to fit
blowfish-compat format.
```
use blowfish_compat::reverse_words;
let mut s = "12345678".to_owned();
reverse_words(unsafe { s.as_bytes_mut() });
assert_eq!(&s, "43218765");
```
 */
#[inline]
pub fn reverse_words(buf: &mut [u8]) {
    #[cfg(target_endian = "little")]
    {
        #[cfg(feature = "bswap")]
        unsafe {
            let buf_len = buf.len();
            // chunk by chunk where size is power of WORD but not huge or "bus-err/out of mem".
            return bswap::u32::swap_memory_inplace(
                &mut buf[0] as *mut u8,
                buf_len - buf_len % WORD,
            );
        }

        #[cfg(not(feature = "bswap"))]
        for chunk in buf.chunks_mut(WORD) {
            chunk.reverse();
        }
    }
}

// copy of the private type-alias `blowfish::Block`.
pub type Block = GenericArray<u8, U8>;

/// BlowfishCompat is wrapper for the `Blowfish`,
/// implements `blowfish::BlockCipher` trait.
#[derive(Clone, Copy)]
pub struct BlowfishCompat {
    inner: Blowfish,
}

impl NewBlockCipher for BlowfishCompat {
    type KeySize = <Blowfish as NewBlockCipher>::KeySize;

    fn new(key: &GenericArray<u8, U56>) -> Self {
        Self {
            inner: <Blowfish as NewBlockCipher>::new(key),
        }
    }

    fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
        <Blowfish as NewBlockCipher>::new_from_slice(key).map(|bf| Self { inner: bf })
    }
}

impl BlockCipher for BlowfishCompat {
    type BlockSize = <Blowfish as BlockCipher>::BlockSize;
    type ParBlocks = <Blowfish as BlockCipher>::ParBlocks;
}

impl BlockEncrypt for BlowfishCompat {
    #[inline]
    fn encrypt_block(&self, block: &mut Block) {
        reverse_words(block);
        self.inner.encrypt_block(block);
        reverse_words(block);
    }
}

impl BlockDecrypt for BlowfishCompat {
    #[inline]
    fn decrypt_block(&self, block: &mut Block) {
        reverse_words(block);
        self.inner.decrypt_block(block);
        reverse_words(block);
    }
}
