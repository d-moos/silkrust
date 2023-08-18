use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use silkrust::net::MessageBuffer;

#[tokio::main]
async fn main() {

    let mut stream = TcpStream::connect("filter.evolin.net:4001").await.expect("asdfasdfasdf");

    let mut message_buffer = MessageBuffer::default();
    let mut netbuffer = [0u8;4096];

    let len = stream.read(&mut netbuffer).await.unwrap();
    let messages = message_buffer.read(netbuffer, len);

    for message in messages {
        println!("{}", message);
    }
}