use silkrust::net::message::{Message, MessageDirection, MessageId, MessageKind};
use silkrust::net::{MessageTable, NetClient, Process};
use std::collections::HashMap;

struct HandshakeReq;
impl Process for HandshakeReq {
    fn process(&mut self, m: Message) {
        println!("hey!");
    }
}

#[tokio::main]
async fn main() {
    let mut m_table: MessageTable = HashMap::new();
    m_table.insert(
        MessageId::new()
            .with_operation(0)
            .with_kind(MessageKind::NetEngine)
            .with_direction(MessageDirection::Req),
        Box::new(HandshakeReq),
    );

    let mut client = NetClient::connect("filter.evolin.net:4001", m_table)
        .await
        .expect("/");

    client.run().await;
}
