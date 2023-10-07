use crate::net::massive::MassiveBuffer;
use crate::net::message::MessageDirection::Req;
use crate::net::message::MessageKind::Framework;
use crate::net::message::{Message, MessageId};
use crate::net::NetConnection;
use crate::security::Security;
use bytes::Buf;
use log::{error, trace};
use queues::{IsQueue, Queue};
use std::collections::HashMap;
use tokio::net::TcpStream;
#[macro_export]

macro_rules! construct_processor_table {
    // use a given instance (used when the processor is stateful)
    ($($kind:ident, $op:expr, $dir:ident = $proc:ident = $instance:expr),* $(,)?) => {
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
    ($($kind:ident, $op:expr, $dir:ident = $proc:ident),* $(,)?) => {
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

pub type Processor = Box<dyn Process + Send>;
pub type MessageTable = HashMap<MessageId, Processor>;

pub trait Process {
    fn process(&mut self, net_client: &mut NetClient, m: Message);
}

pub struct NetClient {
    name: String,
    connection: NetConnection,
    massive_buffer: MassiveBuffer,
    security: Security,
    loopback: Queue<Message>,
}

impl From<TcpStream> for NetClient {
    fn from(value: TcpStream) -> Self {
        let connection: NetConnection = value.into();
        Self {
            connection,
            massive_buffer: MassiveBuffer::default(),
            security: Security::default(),
            name: String::from("Unidentified"),
            loopback: Queue::new(),
        }
    }
}

impl NetClient {
    pub async fn connect(addr: &str) -> std::io::Result<Self> {
        let connection = NetConnection::open(addr).await?;
        Ok(Self {
            connection,
            massive_buffer: MassiveBuffer::default(),
            security: Security::default(),
            name: String::from("Unidentified"),
            loopback: Queue::new(),
        })
    }

    pub fn identify(&mut self, name: &str) {
        self.name = name.to_owned();
        // self.connection.identify(name);
    }

    pub fn close(&mut self) {
        self.connection.close();
    }

    pub fn set_security(&mut self, security: Security) {
        self.security = security;
    }

    pub fn security_mut(&mut self) -> &mut Security {
        &mut self.security
    }

    pub fn process_messages(
        &mut self,
        message_table: &mut MessageTable,
        default_handler: &mut Processor,
        limit: usize,
    ) {
        while let Ok(m) = self.loopback.remove() {
            trace!("IN  {} {}", self.name, m);
            self.process_or_default(message_table, default_handler, m);
        }

        let mut counter = 0;
        while let Some(m) = self.connection.take().unwrap() {
            trace!("IN  {} {}", self.name, m);

            // decrypt
            let m = self.security.decrypt(m);

            // TODO: check error detection

            self.process_or_default(message_table, default_handler, m);

            counter += 1;
            if counter > limit {
                break;
            }
        }
    }

    fn process_or_default(
        &mut self,
        message_table: &mut MessageTable,
        default_handler: &mut Processor,
        m: Message,
    ) {
        if let Some(processor) = message_table.get_mut(m.header().id()) {
            processor.process(self, m);
        } else {
            default_handler.process(self, m);
        }
    }

    pub fn receive(&mut self, message: Message) {
        self.loopback.add(message).expect("never err");
    }

    pub fn send(&mut self, message: Message) {
        let message = self.security.encode(message);

        trace!("OUT {} {}", self.name, message);
        self.connection.put(message).expect("fix");
    }
}
