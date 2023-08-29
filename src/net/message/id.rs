use crate::net::message::direction::MessageDirection;
use crate::net::message::kind::MessageKind;
use bitfield_struct::bitfield;
use std::fmt::{Display, Formatter};

/// The message Id (also known as "Header")
///
/// the message id consists of a 2-bit [MessageDirection], a 2-bit [MessageKind] and a 12-bit
/// `MessageOperation`.
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
#[bitfield(u16)]
#[derive(PartialEq, Eq, Hash)]
pub struct MessageId {
    #[bits(12)]
    pub operation: usize,
    #[bits(2)]
    pub kind: MessageKind,
    #[bits(2)]
    pub direction: MessageDirection,
}

impl Display for MessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let opcode: u16 = self.clone().into();
        write!(
            f,
            "[{} | {} | {}] (0x{:X})",
            self.direction(),
            self.kind(),
            self.operation(),
            opcode
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::net::message::id::MessageId;
    use crate::net::message::{MessageDirection, MessageKind};

    #[test]
    fn back_and_forth_works() {
        let id_from_u16 = MessageId::from(0x5000);
        assert_eq!(id_from_u16.operation(), 0);
        assert_eq!(id_from_u16.direction(), MessageDirection::Req);
        assert_eq!(id_from_u16.kind(), MessageKind::NetEngine);

        let id_from_struct = MessageId::new()
            .with_direction(MessageDirection::Ack)
            .with_kind(MessageKind::NetEngine)
            .with_operation(0);
        assert_eq!(id_from_struct.0, 0x9000);
    }
}
