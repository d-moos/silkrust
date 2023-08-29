use std::time::Duration;
use tokio::time::sleep;
use crate::processor::{
    message_ops::{framework::*, net_engine::*},
    HandshakeReqProcessor, ModuleIdentification, NetPing, ShardListProcessor
};
use silkrust::construct_processor_table;
use silkrust::net::{MessageTable, NetClient, Process};
mod processor;
mod bot;

#[tokio::main]
async fn main() {
    env_logger::builder().init();

    let mut m_table = construct_processor_table! {
            Framework, KEEP_ALIVE, NoDir = NetPing,
            Framework, MODULE_IDENTIFICATION, NoDir = ModuleIdentification,
            Framework, SHARD_LIST, Ack = ShardListProcessor,
            NetEngine, HANDSHAKE, Req = HandshakeReqProcessor
    };

    let mut client = NetClient::connect("filter.evolin.net:4001")
        .await
        .expect("/");

    loop {
        // process messages
        client.process_messages(&mut m_table, 1).await;

        // process bot logic
        // execute each loaded plugin
        println!("bot loop");

        sleep(Duration::from_millis(500)).await;
    }

    println!("done");
}
