use crate::net::message::{Message, MessageId};
use crate::net::NetConnection;
use std::collections::HashMap;
use std::time::Duration;
use bitfield_struct::bitfield;
use bytes::Bytes;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;
use crate::security::Security;

pub type MessageTable = HashMap<MessageId, Box<dyn Process>>;

pub trait Process {
    fn process(&mut self, net_client: &mut NetClient, m: Message);
}

pub struct NetClient {
    connection: NetConnection,
    security: Option<Security>
}

impl NetClient {
    pub async fn connect(addr: &str) -> std::io::Result<Self> {
        let connection = NetConnection::open(addr).await?;
        Ok(Self {
            connection,
            security: None
        })
    }

    pub fn set_security(&mut self, security: Security) {
        self.security = Some(security);
    }

    pub async fn run(&mut self, mut message_table: MessageTable) {
        loop {
            if let Some(m) = self.connection.take().unwrap() {
                // decrypt


                // massive buffer


                // TODO: should a processor be stateful? yes, e.g. Handshake Exchange?
                // no? how can we solve the exchange problem differently? RefCell?
                // store exchange on Client? rather not.
                if let Some(processor) = message_table.get_mut(m.header().id()) {
                    processor.process(self, m);
                } else {
                    println!("unknown messageId");
                }
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    pub fn send(&mut self, message: Message) {
        let message = if let Some(security) = &mut self.security {
            security.encode_new(message)
        } else {
            message
        };

        self.connection.put(message).expect("fix");
        // ...
    }
}
