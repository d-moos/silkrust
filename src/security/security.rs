use crate::security::{Checksum, Sequencer};
use blowfish_compat::BlowfishCompat;

struct Encoder {
    sequencer: Sequencer,
    checksum: Checksum,
}

pub struct Security {
    blowfish: BlowfishCompat,
    encoder: Option<Encoder>,
}

impl Security {
    fn encode(&mut self, data: &[u8]) -> Option<(u8, u8)> {
        self.encoder.map_or(None, |mut e| {
            Some((e.sequencer.next(), e.checksum.compute(data, data.len())))
        })
    }
}
