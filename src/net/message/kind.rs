use std::fmt::{Display, Formatter};

const NONE: u16 = 0;
const NET_ENGINE: u16 = 1;
const FRAMEWORK: u16 = 2;
const GAME: u16 = 3;

/// The message kind
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum MessageKind {
    /// ? examples?
    None,

    /// NetworkEngine
    NetEngine,

    /// Base Framework
    Framework,

    /// Actual Gameplay (Agent interactions, Shard operations, ...)
    Game
}

impl Display for MessageKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MessageKind::None => "None",
            MessageKind::NetEngine => "NetEngine",
            MessageKind::Framework => "Framework",
            MessageKind::Game => "Game",
        })
    }
}

impl MessageKind {
    pub const fn into_bits(self) -> u16 {
        match self {
            MessageKind::None => NONE,
            MessageKind::NetEngine => NET_ENGINE,
            MessageKind::Framework => FRAMEWORK,
            MessageKind::Game => GAME
        }
    }

    pub const fn from_bits(value: u16) -> Self {
        match value {
            NONE => MessageKind::None,
            NET_ENGINE => MessageKind::NetEngine,
            FRAMEWORK => MessageKind::Framework,
            GAME => MessageKind::Game,
            _ => panic!("invalid message kind")
        }
    }
}