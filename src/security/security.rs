use blowfish_compat::BlowfishCompat;
use crate::security::{Checksum, Sequencer};

pub struct Security {
    blowfish: BlowfishCompat,
    sequencer: Option<Sequencer>,
    checksum: Option<Checksum>,
}