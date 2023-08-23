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

        const KEEP_ALIVE: usize = 2;
        net_client.send(Message::new(NoDir, Framework, 2, Bytes::new()));
    }
}
