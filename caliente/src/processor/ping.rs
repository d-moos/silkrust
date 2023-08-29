use crate::processor::message_ops::framework::KEEP_ALIVE;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::{info, trace};
use silkrust::net::message::MessageDirection::NoDir;
use silkrust::net::message::MessageKind::Framework;
use silkrust::net::message::{Header, Message, MessageId};
use silkrust::net::{NetClient, Process};

#[derive(Default)]
pub struct NetPing;

impl Process for NetPing {
    fn process(&mut self, net_client: &mut NetClient, _: Message) {
        trace!("PONG");

        net_client.send(Message::new(NoDir, Framework, KEEP_ALIVE, Bytes::new()));
    }
}
