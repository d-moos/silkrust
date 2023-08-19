use queues::{IsQueue, Queue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::{select, spawn};
use crate::net::message::{Message, MessageId};
use crate::net::NetConnection;

pub type MessageTable = HashMap<MessageId, Box<dyn Process>>;

pub trait Process {
    fn process(&mut self, m: Message);
}

pub struct NetClient {
    message_table: MessageTable,
    connection: NetConnection,
}

impl NetClient {
    pub async fn connect(addr: &str, message_table: MessageTable) -> std::io::Result<Self> {
        let connection = NetConnection::open(addr).await?;
        Ok(Self {
            connection,
            message_table,
        })
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(m) = self.connection.take().unwrap() {
                // decrypt

                // massive buffer

                if let Some(processor) = self.message_table.get_mut(m.header().id()) {
                    processor.process(m);
                } else {
                    println!("unknown messageId");
                }
            }

            sleep(Duration::from_millis(10)).await;
        }
    }
}