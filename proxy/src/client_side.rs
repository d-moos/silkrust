use bytes::{BufMut, Bytes, BytesMut};
use log::{error, info};
use silkrust::construct_processor_table;
use silkrust::net::message::Message;
use silkrust::net::message::MessageDirection::Req;
use silkrust::net::message::MessageKind::NetEngine;
use silkrust::net::net_engine::{
    ErrorDetectionSeed, ExchangeResponse, ExchangeSetup, HandshakeOptions,
};
use silkrust::net::{MessageTable, NetClient, Process, Processor};
use silkrust::security::blowfish_compat::{BlowfishCompat, NewBlockCipher};
use silkrust::security::{Challenge, Exchange, Initiator, NotSet, SecurityBuilder};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, RwLock};
use silkrust::net::io::BytesExtension;

struct ModuleIdentificationProcessor {
    sender: Sender<Message>,
}
impl ModuleIdentificationProcessor {
    fn new(sender: Sender<Message>) -> Self {
        Self { sender }
    }
}

struct HandshakeReqProcessor {
    context_builder: Exchange<NotSet>,
}
impl HandshakeReqProcessor {
    pub fn new(context_builder: Exchange<NotSet>) -> Self {
        Self { context_builder }
    }
}

#[derive(Default)]
struct HandshakeAckProcessor {
    can_receive_forwarded_messages: Arc<RwLock<bool>>,
}

impl HandshakeAckProcessor {
    pub fn new(can_receive_forwarded_messages: Arc<RwLock<bool>>) -> Self {
        Self {
            can_receive_forwarded_messages,
        }
    }
}

struct ServerForwardProcessor {
    sender: Sender<Message>,
}

impl ServerForwardProcessor {
    fn new(sender: Sender<Message>) -> Self {
        Self { sender }
    }
}

impl Process for HandshakeAckProcessor {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        let mut v = self.can_receive_forwarded_messages.write().unwrap();
        *v = true;
        info!("[Handshake 🤝] completed with key exchange ✅!");
    }
}
impl Process for HandshakeReqProcessor {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        let mem: Bytes = m.reader();
        let response = ExchangeResponse::from(mem);

        let exchange = self.context_builder.remote(response.public);
        if let Err(_) = <Initiator as Challenge>::verify(&exchange, response.signature) {
            net_client.close();
            error!("remote signature does not match calculate signature");
            panic!();
        }

        // challenge is ok!
        let challenge = <Initiator as Challenge>::create(&exchange);

        let options = HandshakeOptions::new().with_challenge(true);
        let mut data = BytesMut::new();
        data.put_u8(options.into());
        data.put_slice(challenge.as_slice());

        let m = Message::new(Req, NetEngine, 0, data.freeze());
        net_client.send(m);

        // finalize security
        let final_key = <Initiator as Challenge>::finalize(&exchange);
        let blowfish =
            BlowfishCompat::new_from_slice(&final_key).expect("could not initialize blowfish");

        info!("[Handshake 🤝] 🙋🏽‍♂️ Sending Challenge");
        net_client.security_mut().blowfish = Some(blowfish);
    }
}
impl Process for ServerForwardProcessor {
    fn process(&mut self, _net_client: &mut NetClient, m: Message) {
        self.sender.send(m).expect("channel is not clogged");
    }
}

impl Process for ModuleIdentificationProcessor {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        self.sender.send(m.clone()).expect("channel is not clogged");
        let mut reader = m.reader();
        let name = reader.get_string().unwrap();
        net_client.identify(name.as_str());
    }
}

pub struct ClientSide {
    client_connection: NetClient,
    receiver: Receiver<Message>,
}

impl ClientSide {
    pub fn new(client_connection: NetClient, receiver: Receiver<Message>) -> Self {
        Self {
            client_connection,
            receiver,
        }
    }

    pub fn run(&mut self, sender: Sender<Message>) {
        let exchange = Exchange::default()
            .set_initial(rand::random())
            .set_generator(rand::random::<u32>() & 0x7FFFFFFF)
            .set_prime(rand::random::<u32>() & 0x7FFFFFFF)
            .set_private(rand::random::<u32>() & 0x7FFFFFFF);

        // initiate handshake
        self.init_handshake(&exchange);

        let can_receive_forwarded_messages = Arc::new(RwLock::new(false));

        let handshake_ack_processor =
            HandshakeAckProcessor::new(can_receive_forwarded_messages.clone());
        let mut forwarder: Processor = Box::new(ServerForwardProcessor::new(sender.clone()));
        let mut message_table: MessageTable = construct_processor_table! {
            NetEngine, 0, Ack = HandshakeAckProcessor = handshake_ack_processor,
            NetEngine, 0, Req = HandshakeReqProcessor = HandshakeReqProcessor::new(exchange),
            Framework, 1, NoDir = ModuleIdentificationProcessor = ModuleIdentificationProcessor::new(sender)
        };

        // loop
        loop {
            self.client_connection
                .process_messages(&mut message_table, &mut forwarder, 100);
            if let Ok(v) = can_receive_forwarded_messages.try_read() {
                if *v {
                    // send received messages
                    match self.receiver.try_recv() {
                        /// client message can be sent to server
                        Ok(m) => {
                            self.client_connection.send(m);
                        }

                        /// exit loop if sender has disconnected
                        Err(TryRecvError::Disconnected) => {
                            return;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn init_handshake(&mut self, exchange: &Exchange<NotSet>) {
        let (sequence_seed, checksum_seed): (u32, u32) = (rand::random(), rand::random());

        let security = SecurityBuilder::default()
            .encoding_requirements((true, false))
            .error_detection((sequence_seed, checksum_seed))
            .build();

        let options = HandshakeOptions::new()
            .with_disabled(false)
            .with_challenge(false)
            .with_encryption(false)
            .with_exchange(true)
            .with_error_detection(true);

        self.client_connection.set_security(security);

        // building the message
        let error_detection = ErrorDetectionSeed::new(sequence_seed, checksum_seed);
        let setup = ExchangeSetup::new(
            exchange.get_initial(),
            exchange.get_generator(),
            exchange.get_prime(),
            exchange.get_local(),
        );

        let mut data = BytesMut::new();
        data.put_u8(options.into());
        data.put::<Bytes>(error_detection.into());
        data.put::<Bytes>(setup.into());

        let m = Message::new(Req, NetEngine, 0, data.freeze());
        self.client_connection.send(m);
    }
}
