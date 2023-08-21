const DEFAULT_SEED: u32 = 0x9ABFB3B6;

pub struct Sequencer {
    byte0: u8,
    byte1: u8,
    byte2: u8,
}

impl Sequencer {
    pub fn new(value: u32) -> Self {
        let mut0 = if value != 0 { value } else { DEFAULT_SEED };
        let mut1 = generate_value(mut0);
        let mut2 = generate_value(mut1);
        let mut3 = generate_value(mut2);
        let mut4 = generate_value(mut3);

        let mut byte1: u8 = ((mut1 & 0xFF) ^ (mut2 & 0xFF)) as u8;
        byte1 = if byte1 == 0 { 1 } else { byte1 };

        let mut byte2: u8 = ((mut4 & 0xFF) ^ (mut3 & 0xFF)) as u8;
        byte2 = if byte2 == 0 { 1 } else { byte2 };

        Self {
            byte0: byte2 ^ byte1,
            byte1,
            byte2,
        }
    }

    pub fn next(&mut self) -> u8 {
        let value = (self.byte2 as u32 * (!self.byte0 as u32 + self.byte1 as u32)) as u8;
        self.byte0 = (value ^ value >> 4) as u8;

        self.byte0
    }
}

fn generate_value(mut value: u32) -> u32 {
    for _ in 0..32 {
        let mut v = value;
        v = v >> 2 ^ value;
        v = v >> 2 ^ value;
        v = v >> 1 ^ value;
        v = v >> 1 ^ value;
        v = v >> 1 ^ value;
        value = (((value >> 1) | (value << 31)) & !1u32) | (v & 1);
    }

    value
}

#[cfg(test)]
mod tests {
    use crate::security::sequencer::generate_value;
    use crate::security::Sequencer;

    #[test]
    fn compare_init_with_known_sequencer_impl() {
        let sequencer = Sequencer::new(0x12345678);

        assert_eq!(sequencer.byte0, 129);
        assert_eq!(sequencer.byte1, 114);
        assert_eq!(sequencer.byte2, 243);
    }

    #[test]
    fn compare_next_with_known_sequencer_impl() {
        let mut sequencer = Sequencer::new(0x1234);

        assert_eq!(sequencer.next(), 4);
        assert_eq!(sequencer.next(), 222);
    }

    #[test]
    fn compare_generate_value_with_known_sequencer_impl() {
        let value = 0x12345678;

        let m0 = generate_value(value);
        let m1 = generate_value(m0);

        assert_eq!(m0, 1706579037);
        assert_eq!(m1, 1019020591);
    }
}
