use crate::net::massive::{MassiveBuffer, MassiveError};
use crate::net::message::MessageDirection::Req;
use crate::net::message::MessageKind::Framework;
use crate::net::message::{Header, Message, MessageId};
use crate::net::NetConnection;
use crate::security::Security;
use bitfield_struct::bitfield;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::{error, trace, warn};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;

#[macro_export]
macro_rules! construct_processor_table {
    // use a given instance (used when the processor is stateful)
    ($($kind:ident, $dir:ident, $op:literal = $proc:ident = $instance:expr),* $(,)?) => {
        {
            let mut m_table = MessageTable::new();
            $(
                m_table.insert(
                    $crate::net::message::MessageId::new()
                        .with_operation($op)
                        .with_kind($crate::net::message::MessageKind::$kind)
                        .with_direction($crate::net::message::MessageDirection::$dir),
                    Box::new($instance),
                );
            )*
            m_table
        }
    };

    // use a default implementation (used when the processor is stateless
    ($($kind:ident, $dir:ident, $op:literal = $proc:ident),* $(,)?) => {
        {
            let mut m_table = MessageTable::new();
            $(
                m_table.insert(
                   $crate::net::message::MessageId::new()
                        .with_operation($op)
                        .with_kind($crate::net::message::MessageKind::$kind)
                        .with_direction($crate::net::message::MessageDirection::$dir),
                    Box::new($proc::default()),
                );
            )*
            m_table
        }
    };
}

pub type MessageTable = HashMap<MessageId, Box<dyn Process>>;

pub trait Process {
    fn process(&mut self, net_client: &mut NetClient, m: Message);
}

pub struct NetClient {
    connection: NetConnection,
    massive_buffer: MassiveBuffer,
    security: Option<Security>,
}

impl NetClient {
    pub async fn connect(addr: &str) -> std::io::Result<Self> {
        let connection = NetConnection::open(addr).await?;
        Ok(Self {
            connection,
            massive_buffer: MassiveBuffer::default(),
            security: None,
        })
    }

    pub fn set_security(&mut self, security: Security) {
        self.security = Some(security);
    }

    pub fn security_mut(&mut self) -> &mut Option<Security> {
        &mut self.security
    }

    pub async fn run(&mut self, mut message_table: MessageTable) {
        loop {
            if let Some(m) = self.connection.take().unwrap() {
                // decrypt

                // massive buffer
                if let Some(m) = NetClient::massive_check(m, &mut self.massive_buffer) {
                    if let Some(processor) = message_table.get_mut(m.header().id()) {
                        processor.process(self, m);
                    } else {
                        warn!("no processor found for {}", m.header().id());
                    }
                }
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    fn massive_check(m: Message, massive_buffer: &mut MassiveBuffer) -> Option<Message> {
        if m.header().id()
            != &MessageId::new()
                .with_kind(Framework)
                .with_direction(Req)
                .with_operation(13)
        {
            return Some(m);
        }

        match massive_buffer.add(m) {
            Ok(_) => massive_buffer.collect(),
            Err(e) => {
                error!("could not add massive message to buffer! ({:?})", e);
                None
            }
        }
    }

    pub fn send(&mut self, message: Message) {
        let message = if let Some(security) = &mut self.security {
            security.encode(message)
        } else {
            message
        };

        self.connection.put(message).expect("fix");
        // ...
    }
}
