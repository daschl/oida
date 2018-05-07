use parse::LogLine;
use regex::Regex;
use chrono::NaiveDateTime;

lazy_static! {
    static ref CONNECTING_NODE: Regex = Regex::new(r"Connecting Node ([^/]+)").unwrap();
}
lazy_static! {
    static ref DISCONNECTING_NODE: Regex = Regex::new(r"Removing and disconnecting node ([^/]+)").unwrap();
}
lazy_static! {
    static ref CLIENT_INIT: Regex = Regex::new(r"CouchbaseEnvironment: (.+)").unwrap();
}

#[derive(Debug)]
pub enum Event {
    ConnectingNode(NaiveDateTime, String),
    DisconnectingNode(NaiveDateTime, String),
    ClientInit(NaiveDateTime, String),
}

#[derive(Debug)]
pub struct EventIndex {
    events: Vec<Event>,
}

impl EventIndex {
    pub fn new() -> Self {
        EventIndex { events: vec![] }
    }

    pub fn feed(&mut self, line: LogLine) {
        let message = line.message.expect("No message");

        let cn_caps = CONNECTING_NODE.captures(&message);
        if cn_caps.is_some() {
            let found = cn_caps.unwrap().get(1).unwrap();
            self.events.push(Event::ConnectingNode(
                line.timestamp.expect("no timestamp found"),
                String::from(found.as_str()),
            ));
            return;
        }

        let dn_caps = DISCONNECTING_NODE.captures(&message);
        if dn_caps.is_some() {
            let found = dn_caps.unwrap().get(1).unwrap();
            self.events.push(Event::DisconnectingNode(
                line.timestamp.expect("no timestamp found"),
                String::from(found.as_str()),
            ));
            return;
        }

        let ci_caps = CLIENT_INIT.captures(&message);           

        if ci_caps.is_some() {
            let found = ci_caps.unwrap().get(1).unwrap();
            self.events.push(Event::ClientInit(
                line.timestamp.expect("no timestamp found"),
                String::from(found.as_str()),
            ));
            return;
        }
    }
}
