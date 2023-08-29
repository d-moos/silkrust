use queues::{IsQueue, Queue};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use bytes::Bytes;
use log::trace;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::{select, spawn};
use tokio::time::sleep;
use crate::net::message::{MAX_MESSAGE_SIZE, Message};
use crate::net::MessageBuffer;

type SyncMutex<T> = Arc<Mutex<T>>;
type AsyncMutex<T> = Arc<tokio::sync::Mutex<T>>;
type MessageQueue = Queue<Message>;
type SyncQueue = SyncMutex<MessageQueue>;

pub struct NetConnection {
    addr: String,
    inbound: SyncQueue,
    outbound: SyncQueue,

    run_handle: JoinHandle<()>,
}

impl NetConnection {
    pub async fn open(addr: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let inbound = Arc::new(Mutex::new(Queue::new()));
        let outbound = Arc::new(Mutex::new(Queue::new()));

        let run_handle = spawn(NetConnection::run(stream, inbound.clone(), outbound.clone()));

        Ok(Self {
            addr: addr.to_string(),
            run_handle,
            inbound,
            outbound
        })
    }

    async fn run(mut stream: TcpStream, inbound_queue: SyncQueue, outbound_queue: SyncQueue) {
        let (read, write) = stream.split();

        let inbound = inbound_queue.clone();
        let outbound = outbound_queue.clone();
        let f = select! {
            r = NetConnection::inbound_loop(read, inbound_queue) => r,
            r = NetConnection::outbound_loop(write, outbound_queue) => r,
        };

        println!("select resulted in {:?}", f);
    }

    pub fn close(&mut self) {
        self.run_handle.abort();
    }

    pub fn take(&mut self) -> Result<Option<Message>, PoisonError<MutexGuard<MessageQueue>>> {
        Ok(self
            .inbound
            .lock()?
            .remove()
            .map_or(None, |m| Some(m)))
    }

    pub fn put(&mut self, message: Message) -> Result<(), PoisonError<MutexGuard<MessageQueue>>> {
        let mut queue = self.outbound.lock()?;
        queue.add(message).expect("returns always Ok(None) according to docs");
        Ok(())
    }

    /// TODO: Error Mapping
    async fn inbound_loop<'a>(
        mut stream: ReadHalf<'a>,
        inbound: SyncQueue,
    ) -> Result<(), ()> {
        loop {
            // stream.readable().await.map_err(|_| ())?;
            let mut message_buffer = MessageBuffer::default();

            let mut net_buffer = [0u8; MAX_MESSAGE_SIZE];
            let len = stream.read(&mut net_buffer).await.map_err(|_| ())?;

            if len == 0 {
                return Err(());
            }

            let messages = message_buffer.read(net_buffer, len);
            let mut inbound_queue = inbound.lock().map_err(|_| ())?;
            for message in messages {
                trace!("IN:  {}", message);
                inbound_queue.add(message).map_err(|_| ())?;
            }
        }
    }

    async fn outbound_loop<'a>(mut stream: WriteHalf<'a>, outbound: SyncQueue) -> Result<(), ()> {
        loop {
            let messages = {
                let mut outbound_queue = outbound.lock().expect("fix");

                // todo: set max outbound count
                let mut messages = Vec::<Message>::new();
                while let Ok(message) = outbound_queue.remove() {
                    messages.push(message);
                }

                messages
            };

            for message in messages {
                trace!("OUT: {}", message);
                let b: Bytes = message.into();
                stream.write(b.as_ref()).await.map_err(|_| ())?;
            }

            sleep(Duration::from_millis(10)).await;
        }
    }
}