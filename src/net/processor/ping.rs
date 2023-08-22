use crate::net::message::MessageDirection::NoDir;
use crate::net::message::MessageKind::{Framework, NetEngine};
use crate::net::message::{Header, Message, MessageDirection, MessageId, MessageKind};
use crate::net::{NetClient, Process};
use blowfish_compat::cipher::generic_array::typenum::Mod;
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

struct Module {
    servicename: String,
    is_local: bool,
}

impl From<Bytes> for Module {
    fn from(mut value: Bytes) -> Self {
        let len = value.get_u16_le() as usize;
        let mut name_buffer = vec![0u8; len];
        value.copy_to_slice(&mut name_buffer);
        Self {
            servicename: String::from_utf8(name_buffer).unwrap(),
            is_local: value.get_u8() == 1,
        }
    }
}

impl Into<Bytes> for Module {
    fn into(self) -> Bytes {
        let mut mem = BytesMut::new();

        mem.put_u16_le(self.servicename.len() as u16);
        mem.put_slice(self.servicename.as_bytes());
        mem.put_u8(self.is_local.into());

        mem.freeze()
    }
}

pub struct ModuleIdentification;

impl Process for ModuleIdentification {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        let module = Module::from(m.reader());

        info!("opposite module is {}", module.servicename);

        let response = Module {
            is_local: false,
            servicename: String::from("SR_CLIENT"),
        };

        info!("we are {}", module.servicename);

        let mem: Bytes = response.into();
        net_client.send(Message::new(
            Header::new(
                MessageId::new()
                    .with_operation(1)
                    .with_kind(Framework)
                    .with_direction(NoDir),
                mem.len() as u16,
            ),
            mem,
        ))
    }
}
