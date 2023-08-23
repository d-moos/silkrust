use crate::processor::{
    message_ops::{framework::*, net_engine::*},
    HandshakeReqProcessor, ModuleIdentification, NetPing, ShardListProcessor
};
use silkrust::construct_processor_table;
use silkrust::net::{MessageTable, NetClient, Process};
mod processor;

#[tokio::main]
async fn main() {
    env_logger::builder().init();

    let m_table = construct_processor_table! {
            Framework, KEEP_ALIVE, NoDir = NetPing,
            Framework, MODULE_IDENTIFICATION, NoDir = ModuleIdentification,
            Framework, SHARD_LIST, Ack = ShardListProcessor,
            NetEngine, HANDSHAKE, Req = HandshakeReqProcessor
    };

    let mut client = NetClient::connect("filter.evolin.net:4001")
        .await
        .expect("/");

    client.run(m_table).await;

    println!("done");
}
