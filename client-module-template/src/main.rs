use silkrust::net::message::{Header, Message, MessageDirection, MessageId, MessageKind};
use silkrust::net::{MessageTable, NetClient, Process};
use std::collections::HashMap;
use silkrust::construct_processor_table;
use crate::processor::{HandshakeReqProcessor, ModuleIdentification, NetPing};


mod processor;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .init();

    let m_table = construct_processor_table! {
            Framework, NoDir, 2 = NetPing,
            Framework, NoDir, 1 = ModuleIdentification,
            NetEngine, Req, 0 = HandshakeReqProcessor
    };

    let mut client = NetClient::connect("filter.evolin.net:4001")
        .await
        .expect("/");

    client.run(m_table).await;

    println!("done");
}
