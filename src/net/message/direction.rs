use std::fmt::{Display, Formatter};

const NO_DIR: u8 = 0;
const REQ: u8 = 1;
const ACK: u8 = 2;

/// Message Direction
#[derive(Clone, Debug)]
pub enum MessageDirection {
    /// No Direction
    ///
    /// The message is sent as part of a fire-and-forget operation
    NoDir,

    /// Request
    ///
    /// The message expects to receive (at some point) a reply.
    Req,

    /// Acknowledge
    ///
    /// The message is sent as answer to a previous message
    Ack
}

impl Display for MessageDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MessageDirection::NoDir => "NoDir",
            MessageDirection::Req => "Req",
            MessageDirection::Ack => "Ack",
        })
    }
}

impl From<u8> for MessageDirection {
    fn from(value: u8) -> Self {
        match value {
            NO_DIR => MessageDirection::NoDir,
            REQ => MessageDirection::Req,
            ACK => MessageDirection::Ack,
            _ => panic!("invalid message direction")
        }
    }
}

impl Into<u8> for MessageDirection {
    fn into(self) -> u8 {
        match self {
            MessageDirection::NoDir => NO_DIR,
            MessageDirection::Req => REQ,
            MessageDirection::Ack => ACK,
        }
    }
}