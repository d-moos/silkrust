use std::fmt::{Display, Formatter};

const NONE: u8 = 0;
const NET_ENGINE: u8 = 1;
const FRAMEWORK: u8 = 2;
const GAME: u8 = 3;

/// The message kind
#[derive(Clone)]
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

impl Into<u8> for MessageKind {
    fn into(self) -> u8 {
        match self {
            MessageKind::None => NONE,
            MessageKind::NetEngine => NET_ENGINE,
            MessageKind::Framework => FRAMEWORK,
            MessageKind::Game => GAME
        }
    }
}

impl From<u8> for MessageKind {
    fn from(value: u8) -> Self {
        match value {
            NONE => MessageKind::None,
            NET_ENGINE => MessageKind::NetEngine,
            FRAMEWORK => MessageKind::Framework,
            GAME => MessageKind::Game,
            _ => panic!("invalid message kind")
        }
    }
}