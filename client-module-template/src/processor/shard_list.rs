use bytes::{Buf, Bytes};
use silkrust::net::io::{BytesExtension, Fragment};
use silkrust::net::message::Message;
use silkrust::net::{NetClient, Process};
use std::mem::size_of;
use std::string::FromUtf8Error;
use log::{info, log};

#[derive(Default)]
pub struct ShardListProcessor;

impl Fragment for Shard {
    fn get(reader: &mut Bytes) -> Self {
        let id = reader.get_u16_le();
        let name = reader.get_string().expect("todo");
        let clients_connected = reader.get_u16_le();
        let capacity = reader.get_u16_le();
        let operating = reader.get_u8() == 1;
        let farm_id = reader.get_u8();

        Shard {
            id,
            name,
            clients_connected,
            capacity,
            operating,
            farm_id,
        }
    }
}

struct Shard {
    id: u16,
    name: String,
    clients_connected: u16,
    capacity: u16,
    operating: bool,
    farm_id: u8,
}

struct Farm {
    id: u8,
    name: String,
}

impl Fragment for Farm {
    fn get(reader: &mut Bytes) -> Self {
        let id = reader.get_u8();
        let name = reader.get_string().expect("todo");

        Farm { id, name }
    }
}

impl Process for ShardListProcessor {
    fn process(&mut self, _net_client: &mut NetClient, m: Message) {
        let reader = &mut m.reader();
        let farms = reader.get_collection::<Farm>();
        let shards = reader.get_collection::<Shard>();

        for farm in farms {
            info!("{}, {}", farm.id, farm.name);
        }

        for shard in shards {
            info!(
                "{}, {}, {}, {}, {}",
                shard.id, shard.name, shard.clients_connected, shard.capacity, shard.operating
            );
        }
    }
}
