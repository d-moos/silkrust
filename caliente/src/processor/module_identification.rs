use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::info;
use silkrust::net::message::MessageDirection::{NoDir, Req};
use silkrust::net::message::MessageKind::Framework;
use silkrust::net::message::{Header, Message, MessageId};
use silkrust::net::{NetClient, Process};
use silkrust::net::io::BytesExtension;
use crate::processor::message_ops::framework::{MODULE_IDENTIFICATION, SHARD_LIST};

struct Module {
    servicename: String,
    is_local: bool,
}

impl From<Bytes> for Module {
    fn from(mut value: Bytes) -> Self {
        Self {
            servicename: value.get_string().expect("todo"),
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

#[derive(Default)]
pub struct ModuleIdentification;

impl Process for ModuleIdentification {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        let module = Module::from(m.reader());

        info!("opposing module is {}", module.servicename);

        let response = Module {
            is_local: false,
            servicename: String::from("SR_CLIENT"),
        };

        let mem: Bytes = response.into();

        net_client.send(Message::new(NoDir, Framework, MODULE_IDENTIFICATION, mem));
        net_client.send(Message::new(Req, Framework, SHARD_LIST, Bytes::new()));
    }
}
