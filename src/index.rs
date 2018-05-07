use parse::LogLine;
use regex::Regex;
use chrono::NaiveDateTime;
use serde_cbor::{from_reader, to_writer};
use std::io::{Read, Write};

lazy_static! {
    static ref CONNECTING_NODE: Regex = Regex::new(r"Connecting Node ([^/]+)").unwrap();
}
lazy_static! {
    static ref DISCONNECTING_NODE: Regex =
        Regex::new(r"Removing and disconnecting node ([^/]+)").unwrap();
}
lazy_static! {
    static ref CLIENT_INIT: Regex = Regex::new(r"CouchbaseEnvironment: (.+)").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TopologyEvent {
    ConnectingNode(NaiveDateTime, String),
    DisconnectingNode(NaiveDateTime, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientEvent {
    ClientInit(NaiveDateTime, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventIndex {
    pub topo_events: Vec<TopologyEvent>,
    pub client_events: Vec<ClientEvent>,
}

impl EventIndex {
    pub fn new() -> Self {
        EventIndex {
            topo_events: vec![],
            client_events: vec![],
        }
    }

    pub fn from_reader<R: Read>(reader: R) -> Self {
        from_reader(reader).expect("Could not read CBOR")
    }

    pub fn feed(&mut self, line: LogLine) {
        let message = line.message.expect("No message");

        let cn_caps = CONNECTING_NODE.captures(&message);
        if cn_caps.is_some() {
            let found = cn_caps.unwrap().get(1).unwrap();
            self.topo_events.push(TopologyEvent::ConnectingNode(
                line.timestamp.expect("no timestamp found"),
                String::from(found.as_str()),
            ));
            return;
        }

        let dn_caps = DISCONNECTING_NODE.captures(&message);
        if dn_caps.is_some() {
            let found = dn_caps.unwrap().get(1).unwrap();
            self.topo_events.push(TopologyEvent::DisconnectingNode(
                line.timestamp.expect("no timestamp found"),
                String::from(found.as_str()),
            ));
            return;
        }

        let ci_caps = CLIENT_INIT.captures(&message);

        if ci_caps.is_some() {
            let found = ci_caps.unwrap().get(1).unwrap();
            self.client_events.push(ClientEvent::ClientInit(
                line.timestamp.expect("no timestamp found"),
                String::from(found.as_str()),
            ));
            return;
        }
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) {
        to_writer(writer, &self).expect("Could not serialize into CBOR!")
    }
}
