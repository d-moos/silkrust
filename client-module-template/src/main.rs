use silkrust::net::message::{Header, Message, MessageDirection, MessageId, MessageKind};
use silkrust::net::{MessageTable, NetClient, Process};
use std::collections::HashMap;
use silkrust::net::processor::HandshakeReqProcessor;

#[tokio::main]
async fn main() {
    let mut m_table: MessageTable = HashMap::new();
    m_table.insert(
        MessageId::new()
            .with_operation(0)
            .with_kind(MessageKind::NetEngine)
            .with_direction(MessageDirection::Req),
        Box::new(HandshakeReqProcessor::default()),
    );

    let mut client = NetClient::connect("filter.evolin.net:4001")
        .await
        .expect("/");

    client.run(m_table).await;

    println!("done");
}
