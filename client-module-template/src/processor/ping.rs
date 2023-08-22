use silkrust::net::message::MessageDirection::NoDir;
use silkrust::net::message::MessageKind::Framework;
use silkrust::net::message::{Header, Message, MessageId};
use silkrust::net::{NetClient, Process};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::{info, trace};

pub struct NetPing;

impl Process for NetPing {
    fn process(&mut self, net_client: &mut NetClient, _: Message) {
        // just response with a pong
        trace!("PONG");
        net_client.send(Message::new(
            Header::new(
                MessageId::new()
                    .with_kind(Framework)
                    .with_direction(NoDir)
                    .with_operation(2),
                0,
            ),
            Bytes::new(),
        ));
    }
}