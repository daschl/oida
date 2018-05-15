use chrono::NaiveDateTime;
use parse::LogLine;
use regex::Regex;
use serde_cbor::{from_reader, to_writer};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::iter::FromIterator;

lazy_static! {
    static ref CONNECTING_NODE: Regex = Regex::new(r"Connecting Node ([^/]+)").unwrap();
}
lazy_static! {
    static ref DISCONNECTING_NODE: Regex =
        Regex::new(r"Removing and disconnecting node ([^/]+)").unwrap();
}
lazy_static! {
    static ref CLIENT_INIT: Regex = Regex::new(r"CouchbaseEnvironment: \{(.+)\}").unwrap();
}

type Hostname = String;
type Bucket = String;

#[derive(Debug, Serialize, Deserialize)]
pub enum TopologyEvent {
    ConnectingNode(NaiveDateTime, Hostname),
    DisconnectingNode(NaiveDateTime, Hostname),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientEvent {
    ClientInit(NaiveDateTime, HashMap<String, String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorEvent {
    CarrierRefreshFailed(NaiveDateTime, Bucket, Hostname),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventIndex {
    pub topo_events: Vec<TopologyEvent>,
    pub client_events: Vec<ClientEvent>,
    pub error_events: Vec<ErrorEvent>,
}

impl EventIndex {
    pub fn new() -> Self {
        EventIndex {
            topo_events: vec![],
            client_events: vec![],
            error_events: vec![],
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
            let converted = self.extract_client_details(found.as_str());
            self.client_events.push(ClientEvent::ClientInit(
                line.timestamp.expect("no timestamp found"),
                converted,
            ));
            return;
        }
    }

    fn extract_client_details(&self, input: &str) -> HashMap<String, String> {
        lazy_static! {
            static ref KV_MAP_R: Regex = Regex::new(r"^[a-zA-Z]+=").unwrap();
        };

        let mut lines = vec![];
        let mut offset = 0;
        while let Some(i) = input[offset..].find(',') {
            let line = &input[offset..offset + i];
            lines.push(line.trim());
            offset = offset + i + 1;
        }

        let mut pairs: Vec<(String, String)> = vec![];
        for line in lines {
            if line.ends_with('}') || !KV_MAP_R.is_match(&line) {
                match pairs.last_mut() {
                    Some(v) => v.1.push_str(line),
                    None => (),
                }
            } else {
                let parts: Vec<&str> = line.split('=').collect();
                pairs.push((parts[0].into(), parts[1].into()));
            }
        }

        HashMap::from_iter(pairs.into_iter())
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) {
        to_writer(writer, &self).expect("Could not serialize into CBOR!")
    }
}
