use bytes::{Buf, Bytes};
use log::info;
use silkrust::construct_processor_table;
use silkrust::net::message::Message;
use silkrust::net::message::MessageDirection::{Ack, Req};
use silkrust::net::message::MessageKind::NetEngine;
use silkrust::net::net_engine::{
    ErrorDetectionSeed, ExchangeResponse, ExchangeSetup, HandshakeOptions,
};
use silkrust::net::{MessageTable, NetClient, Process, Processor};
use silkrust::security::blowfish_compat::{BlowfishCompat, NewBlockCipher};
use silkrust::security::{Challenge, Exchange, Key, Responder, SecurityBuilder, Set, Signature};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use silkrust::net::io::BytesExtension;

struct ModuleIdentificationProcessor {
    sender: Sender<Message>,
}

impl ModuleIdentificationProcessor {
    fn new(sender: Sender<Message>) -> Self {
        Self {
            sender
        }
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

#[derive(Default)]
struct ResponderHandshakeReqProcessor {
    exchange: Exchange<Set>,
}

impl Process for ResponderHandshakeReqProcessor {
    fn process(&mut self, net_client: &mut NetClient, m: Message) {
        let mut reader = m.reader();

        let options = HandshakeOptions::from(reader.get_u8());
        if options.challenge() {
            self.handle_challenge(reader, net_client);
        } else {
            self.handle_setup(options, reader, net_client);
        }
    }
}

impl ResponderHandshakeReqProcessor {
    fn handle_challenge(&mut self, mut reader: Bytes, net_client: &mut NetClient) {
        let mut signature = Signature::default();
        reader.copy_to_slice(&mut signature);

        if let Err(_) = <Responder as Challenge>::verify(&self.exchange, signature) {
            panic!("signature mismatch");
        }

        // update security with final blowfish instance
        let key = <Responder as Challenge>::finalize(&self.exchange);
        let blowfish = BlowfishCompat::new_from_slice(key.as_slice())
            .expect("after exchange key should be valid");
        net_client.security_mut().blowfish = Some(blowfish);

        info!("[Handshake 🤝] completed with key exchange ✅!");

        net_client.send(Message::new(Ack, NetEngine, 0, Bytes::new()));
    }

    fn handle_setup(
        &mut self,
        options: HandshakeOptions,
        mut reader: Bytes,
        net_client: &mut NetClient,
    ) {
        let mut security_builder = SecurityBuilder::default();

        info!("[Handshake 🤝] 🙋🏽‍♂️ Setting Up");

        if options.encryption() {
            let mut key = Key::default();
            reader.copy_to_slice(&mut key);
            security_builder = security_builder.blowfish(key);

            info!("[Handshake 🤝] (in-)secure blowfish initialized");
        }

        if options.error_detection() {
            let error_detection = ErrorDetectionSeed::from(reader.copy_to_bytes(8));
            security_builder = security_builder
                .encoding_requirements((false, true))
                .error_detection((error_detection.sequence, error_detection.checksum));

            info!("[Handshake 🤝] error detection initialized");
        }

        net_client.set_security(security_builder.build());

        let message = if options.exchange() {
            let setup = ExchangeSetup::from(reader.copy_to_bytes(20));

            self.exchange = Exchange::default()
                .set_initial(setup.initial_key)
                .set_generator(setup.generator)
                .set_prime(setup.prime)
                .set_private(rand::random())
                .remote(setup.public);

            let signature = <Responder as Challenge>::create(&self.exchange);
            let response = ExchangeResponse::new(self.exchange.get_local(), signature);

            let mem: Bytes = response.into();
            info!("[Handshake 🤝] responded to key exchange setup!");
            Message::new(Req, NetEngine, 0, mem)
        } else {
            info!("[Handshake 🤝] completed without key exchange ✅!");
            Message::new(Ack, NetEngine, 0, Bytes::new())
        };

        net_client.send(message);
    }
}

struct ClientForwardProcessor {
    sender: Sender<Message>,
}

impl ClientForwardProcessor {
    fn new(sender: Sender<Message>) -> Self {
        Self {
            sender
        }
    }
}

impl Process for ClientForwardProcessor {
    fn process(&mut self, _net_client: &mut NetClient, m: Message) {
        self.sender.send(m).expect("channel is not clogged");
    }
}

pub struct ServerSide {
    /// sends messages to the client-side queue
    // sender: Sender<Message>,

    /// receives message from the client-side queue
    receiver: Receiver<Message>,

    /// connection to the server NetEngine (e.g. GatewayServer, AgentServer, ...)
    server_connection: NetClient,
}

impl ServerSide {
    pub fn new(server_connection: NetClient, receiver: Receiver<Message>) -> Self {
        Self {
            server_connection,
            receiver,
        }
    }

    pub fn run(&mut self, sender: Sender<Message>) {
        let mut message_table: MessageTable = construct_processor_table! {
            Framework, 1, NoDir = ModuleIdentificationProcessor = ModuleIdentificationProcessor::new(sender.clone()),
            NetEngine, 0, Req = ResponderHandshakeReqProcessor = ResponderHandshakeReqProcessor::default()
        };

        let mut forwarder: Processor = Box::new(ClientForwardProcessor::new(sender.clone()));

        loop {
            // process server messages
            self.server_connection
                .process_messages(&mut message_table, &mut forwarder, 100);

            // send received messages
            match self.receiver.try_recv() {
                /// client message can be sent to server
                Ok(m) => {
                    self.server_connection.send(m);
                },

                /// exit loop if sender has disconnected
                Err(TryRecvError::Disconnected) => {
                    return;
                }
                _ => {}
            }
        }
    }
}
