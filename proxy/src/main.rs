use bytes::BufMut;
use log::info;
use silkrust::net::message::Message;
use silkrust::net::{NetClient, Process};
use silkrust::security::blowfish_compat::NewBlockCipher;
use silkrust::security::Challenge;
use std::sync::mpsc;
use std::thread;
use tokio::net::TcpListener;
use crate::client_side::ClientSide;
use crate::server_side::ServerSide;

mod client_side;
mod server_side;

#[tokio::main]
async fn main() {
    // todo move to config
    let listener_endpoint: String = String::from("0.0.0.0:1234");
    let remote_endpoint: String = String::from("filter.evolin.net:4001");

    env_logger::builder().init();

    let server_connection = NetClient::connect(remote_endpoint.as_str())
        .await
        .expect("could not connect to remote");

    let (server_send, server_receive) = mpsc::channel::<Message>();
    let (client_send, client_receive) = mpsc::channel::<Message>();

    let mut server_side = ServerSide::new(server_connection, client_receive);
    let server_handle = thread::Builder::new().stack_size(1024 * 1024 * 4).spawn(move || server_side.run(server_send)).unwrap();


    let listener = TcpListener::bind(listener_endpoint.as_str()).await.expect("could not start listening");
    let (stream, _addr) = listener.accept().await.unwrap();
    let client: NetClient = stream.into();

    let mut client_side = ClientSide::new(client, server_receive);
    let client_handle = thread::Builder::new().stack_size(1024 * 1024 * 8).spawn(move || client_side.run(client_send)).unwrap();

    if let Ok(()) = server_handle.join() {
        info!("server handle ok");
    } else {
        info!("server handle not ok");
    }

    if let Ok(()) = client_handle.join() {
        info!("client handle ok");
    } else {
        info!("client handle not ok");
    }
}