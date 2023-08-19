use queues::{IsQueue, Queue};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{ReadHalf};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::{select, spawn};
use crate::net::message::{MAX_MESSAGE_SIZE, Message};
use crate::net::MessageBuffer;

type SyncMutex<T> = Arc<Mutex<T>>;
type AsyncMutex<T> = Arc<tokio::sync::Mutex<T>>;
type MessageQueue = Queue<Message>;
type SyncQueue = SyncMutex<MessageQueue>;

pub struct NetConnection {
    addr: String,
    inbound_queue: SyncQueue,

    run_handle: JoinHandle<()>,
}

impl NetConnection {
    pub async fn open(addr: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let inbound_queue = Arc::new(Mutex::new(Queue::new()));

        let run_handle = spawn(NetConnection::run(stream, inbound_queue.clone()));

        Ok(Self {
            addr: addr.to_string(),
            run_handle,
            inbound_queue,
        })
    }

    async fn run(mut stream: TcpStream, inbound_queue: SyncQueue) {
        let (read, _) = stream.split();

        let q = inbound_queue.clone();
        select! {
            _ = NetConnection::inbound_loop(read, q) => {}
        }
    }

    pub fn close(&mut self) {
        self.run_handle.abort();
    }

    pub fn take(&mut self) -> Result<Option<Message>, PoisonError<MutexGuard<MessageQueue>>> {
        Ok(self
            .inbound_queue
            .lock()?
            .remove()
            .map_or(None, |m| Some(m)))
    }

    /// TODO: Error Mapping
    async fn inbound_loop<'a>(
        mut stream: ReadHalf<'a>,
        inbound_queue: SyncQueue,
    ) -> Result<(), ()> {
        loop {
            println!("inbound loop");
            stream.readable().await.map_err(|_| ())?;
            let mut message_buffer = MessageBuffer::default();

            let mut net_buffer = [0u8; MAX_MESSAGE_SIZE];
            let len = stream.read(&mut net_buffer).await.map_err(|_| ())?;

            if len == 0 {
                return Err(());
            }

            let messages = message_buffer.read(net_buffer, len);
            let mut inbound_queue = inbound_queue.lock().map_err(|_| ())?;
            for message in messages {
                inbound_queue.add(message).map_err(|_| ())?;
            }
        }
    }
}