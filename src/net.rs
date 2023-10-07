/// TODO: document what a message exactly is, what it consists of and general caveats
pub mod message;

pub use self::message_buffer::MessageBuffer;
mod message_buffer;

pub use self::net_connection::NetConnection;
mod net_connection;

pub use self::net_client::{MessageTable, NetClient, Process, Processor};
mod net_client;

mod massive;

pub mod io;
pub mod net_engine;
