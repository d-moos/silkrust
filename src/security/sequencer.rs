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

        let mut byte2: u8 = ((mut3 & 0xFF) ^ (mut4 & 0xFF)) as u8;
        byte2 = if byte2 == 0 { 1 } else { byte2 };

        Self {
            byte0: byte2 ^ byte1,
            byte1,
            byte2
        }
    }

    pub fn next(&mut self) -> u8 {
        let value = self.byte2 as u32 * (!self.byte0 as u32 + self.byte1 as u32);
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