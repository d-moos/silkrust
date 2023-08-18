use std::fmt::{Display, Formatter};
use crate::net::message::kind::MessageKind;
use crate::net::message::direction::MessageDirection;

const OPERATION_SIZE: u16 = 12;
const OPERATION_OFFSET: u16 = 0;
const OPERATION_MASK: u16 = ((1 << OPERATION_SIZE) - 1) << OPERATION_OFFSET;

const TYPE_SIZE: u16 = 2;
const TYPE_OFFSET: u16 = OPERATION_OFFSET + OPERATION_SIZE;
const TYPE_MASK: u16 = ((1 << TYPE_SIZE) - 1) << TYPE_OFFSET;

const DIRECTION_SIZE: u16 = 2;
const DIRECTION_OFFSET: u16 = TYPE_OFFSET + TYPE_SIZE;
const DIRECTION_MASK: u16 = ((1 << DIRECTION_SIZE) - 1) << DIRECTION_OFFSET;

pub type MessageOperation = u16;

/// The message Id (also known as "Header")
///
/// the message id consists of a 2-bit [MessageDirection], a 2-bit [MessageKind] and a 12-bit
/// [MessageOperation].
///
/// ```text
///  MSB                                           LSB
///  ┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐
///  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
///  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
///  └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
///   15 14 13 12 11 10 09 08 07 06 05 04 03 02 01 00
///    ───   ───   ─────────────────────────────────
///
///     │     │                    │
///     ▼     │                    ▼
/// Direction │                Operation
///           │
///           ▼
///          Kind
/// ```
#[derive(Clone)]
pub struct MessageId {
    pub direction: MessageDirection,
    pub category: MessageKind,
    pub operation: MessageOperation,
}

impl Display for MessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let opcode: u16 = self.clone().into();
        write!(
            f,
            "[{} | {}], {} (0x{:X})",
            self.direction, self.category, self.operation, opcode
        )
    }
}

impl From<u16> for MessageId {
    fn from(value: u16) -> Self {
        MessageId {
            direction: MessageDirection::from(((value & DIRECTION_MASK) >> DIRECTION_OFFSET) as u8),
            category: MessageKind::from(((value & TYPE_MASK) >> TYPE_OFFSET) as u8),
            operation: ((value & OPERATION_MASK) >> OPERATION_OFFSET),
        }
    }
}

impl Into<u16> for MessageId {
    fn into(self) -> u16 {
        let mut value: u16 = 0;
        value = (value & !DIRECTION_MASK)
            | (((self.direction as u16) << DIRECTION_OFFSET) & DIRECTION_MASK);
        value = (value & !TYPE_MASK)
            | (((Into::<u8>::into(self.category) as u16) << TYPE_OFFSET) & TYPE_MASK);
        value =
            (value & !OPERATION_MASK) | (((self.operation) << OPERATION_OFFSET) & OPERATION_MASK);
        value
    }
}

#[cfg(test)]
mod tests {
    use crate::net::message::id::MessageId;

    #[test]
    fn back_and_forth_works() {
        let opcode: u16 = 0x9000;
        let id = MessageId::from(opcode);

        let new_opcode: u16 = id.into();
        assert_eq!(new_opcode, opcode);
    }
}
