use crate::net::message::{Header, Message, MessageId};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::{error, trace};
use crate::net::{NetClient, Process};

#[derive(Default)]
pub struct MassiveProcessor {
    buffer: MassiveBuffer
}

impl Process for MassiveProcessor {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        let collected = match self.buffer.add(m) {
            Ok(_) => self.buffer.collect(),
            Err(e) => {
                error!("could not add massive message to buffer! ({:?})", e);
                None
            }
        };

        if let Some(message) = collected {
            net_client.receive(message);
        }
    }
}

#[derive(Default)]
pub(crate) struct MassiveBuffer {
    header: Option<MassiveHeader>,
    count: usize,
    data: BytesMut,
}


#[derive(Debug)]
pub(crate) enum MassiveError {
    /// Cannot add a Header [Message] to the buffer as it has already been initialized
    AlreadyInitialized,

    /// Cannot add a Body [Message] to the buffer as it requires an initial header message first
    HeaderMissing,

    /// Cannot add another Body [Message] to the buffer as the buffer capacity has been reached.
    TooMany,
}

enum MassiveMessage {
    Header(MassiveHeader),
    Body(MassiveBody),
}

#[derive(Copy, Clone)]
struct MassiveHeader {
    total_count: usize,
    id: MessageId,
}
struct MassiveBody {
    data: Bytes,
}

impl From<Bytes> for MassiveMessage {
    fn from(mut value: Bytes) -> Self {
        if value.get_u8() == 1 {
            MassiveMessage::Header(MassiveHeader {
                total_count: value.get_u16_le() as usize,
                id: MessageId::from(value.get_u16_le()),
            })
        } else {
            MassiveMessage::Body(MassiveBody { data: value })
        }
    }
}

impl MassiveBuffer {
    pub fn add(&mut self, message: Message) -> Result<(), MassiveError> {
        let reader = message.reader();
        let massive = MassiveMessage::from(reader);

        match massive {
            MassiveMessage::Header(header) => self.add_header(header),
            MassiveMessage::Body(body) => self.add_body(body),
        }
    }

    pub fn collect(&mut self) -> Option<Message> {
        let header = self.header?;

        if header.total_count == self.count {
            let data = self.data.clone();
            self.reset();

            trace!("massive collected into: {}", header.id);

            Some(Message::from((
                Header::new(header.id, data.len() as u16),
                data.freeze(),
            )))
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.header = None;
        self.count = 0;
        self.data = BytesMut::new();
    }

    fn add_header(&mut self, header: MassiveHeader) -> Result<(), MassiveError> {
        if let Some(_) = self.header {
            Err(MassiveError::AlreadyInitialized)
        } else {
            trace!("initialized massive buffer for id {}", header.id);
            self.header = Some(header);
            Ok(())
        }
    }

    fn add_body(&mut self, body: MassiveBody) -> Result<(), MassiveError> {
        let header = self.header.as_mut().ok_or(MassiveError::HeaderMissing)?;
        if header.total_count < self.count + 1 {
            Err(MassiveError::TooMany)
        } else {
            self.count += 1;
            self.data.put_slice(&body.data);
            trace!("added body for massive with id {}", header.id);
            Ok(())
        }
    }
}
