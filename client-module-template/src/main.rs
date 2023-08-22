use silkrust::net::message::{Header, Message, MessageDirection, MessageId, MessageKind};
use silkrust::net::{MessageTable, NetClient, Process};
use std::collections::HashMap;
use crate::processor::{HandshakeReqProcessor, ModuleIdentification, NetPing};


mod processor;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .init();
    let mut m_table: MessageTable = HashMap::new();
    m_table.insert(
        MessageId::new()
            .with_operation(0)
            .with_kind(MessageKind::NetEngine)
            .with_direction(MessageDirection::Req),
        Box::new(HandshakeReqProcessor::default()),
    );

    m_table.insert(
        MessageId::new()
            .with_operation(2)
            .with_kind(MessageKind::Framework)
            .with_direction(MessageDirection::NoDir),
        Box::new(NetPing),
    );

    m_table.insert(
        MessageId::new()
            .with_operation(1)
            .with_kind(MessageKind::Framework)
            .with_direction(MessageDirection::NoDir),
        Box::new(ModuleIdentification),
    );



    let mut client = NetClient::connect("filter.evolin.net:4001")
        .await
        .expect("/");

    client.run(m_table).await;

    println!("done");
}
