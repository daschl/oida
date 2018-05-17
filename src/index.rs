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
    ClientInit(NaiveDateTime, ClientSettings),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientSettings {
    inner: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum DefaultSettingCheckVariant {
    Exact(&'static str),
    Contains(&'static str),
}

use self::DefaultSettingCheckVariant::*;

lazy_static! {

        static ref CLIENT_SETTING_DEFAULTS: HashMap<&'static str, DefaultSettingCheckVariant> = {
            let mut m = HashMap::new();
            m.insert("sslEnabled", Exact("false"));
            m.insert("sslKeystoreFile", Exact("'null'"));
            m.insert("sslTruststoreFile", Exact("'null'"));
            m.insert("sslKeystorePassword", Exact("false"));
            m.insert("sslTruststorePassword", Exact( "false"));
            m.insert("sslKeystore", Exact("null"));
            m.insert("sslTruststore", Exact("null"));
            m.insert("bootstrapHttpEnabled", Exact("true"));
            m.insert("bootstrapCarrierEnabled", Exact("true"));
            m.insert("bootstrapHttpDirectPort", Exact("8091"));
            m.insert("bootstrapHttpSslPort", Exact("18091"));
            m.insert("bootstrapCarrierDirectPort", Exact("11210"));
            m.insert("bootstrapCarrierSslPort", Exact("11207"));
            m.insert("responseBufferSize", Exact("16384"));
            m.insert("requestBufferSize", Exact("16384"));
            m.insert("kvServiceEndpoints", Exact( "1"));
            m.insert("viewServiceEndpoints", Exact("12"));
            m.insert("queryServiceEndpoints", Exact("12"));
            m.insert("searchServiceEndpoints", Exact("12"));
            m.insert("configPollInterval", Exact("2500"));
            m.insert("configPollFloorInterval", Exact("50"));
            m.insert("ioPool", Exact("NioEventLoopGroup"));
            m.insert("kvIoPool", Exact("null"));
            m.insert("viewIoPool", Exact("null"));
            m.insert("searchIoPool", Exact("null"));
            m.insert("queryIoPool", Exact("null"));
            m.insert("coreScheduler", Exact("CoreScheduler"));
            m.insert(
                "memcachedHashingStrategy",
Exact("DefaultMemcachedHashingStrategy"),
            );
            m.insert("eventBus", Exact("DefaultEventBus"));
            m.insert("maxRequestLifetime", Exact("75000"));
            m.insert(
                "retryDelay",
Exact("ExponentialDelay{growBy 1.0 MICROSECONDSpowers of 2; lower=100upper=100000}")
            );
            m.insert(
                "reconnectDelay",
Exact("ExponentialDelay{growBy 1.0 MILLISECONDSpowers of 2; lower=32upper=4096}")
            );
            m.insert(
                "observeIntervalDelay",
Exact("ExponentialDelay{growBy 1.0 MICROSECONDSpowers of 2; lower=10upper=100000}")
            );
            m.insert("keepAliveInterval", Exact("30000"));
            m.insert("continuousKeepAliveEnabled", Exact("true"));
            m.insert("keepAliveErrorThreshold", Exact("4"));
            m.insert("keepAliveTimeout", Exact("2500"));
            m.insert("autoreleaseAfter", Exact("2000"));
            m.insert("bufferPoolingEnabled", Exact("true"));
            m.insert("tcpNodelayEnabled", Exact("true"));
            m.insert("mutationTokensEnabled", Exact("false"));
            m.insert("socketConnectTimeout", Exact("1000"));
            m.insert("callbacksOnIoPool", Exact("false"));
            m.insert("disconnectTimeout", Exact("25000"));
            m.insert(
                "requestBufferWaitStrategy",
Contains(".DefaultCoreEnvironment")
            );
            m.insert("certAuthEnabled", Exact("false"));
            m.insert("coreSendHook", Exact("null"));
            m.insert("forceSaslPlain", Exact("false"));
            m.insert("compressionMinRatio", Exact("0.83"));
            m.insert("compressionMinSize", Exact("32"));
            m.insert("queryTimeout", Exact("75000"));
            m.insert("viewTimeout", Exact("75000"));
            m.insert("searchTimeout", Exact("75000"));
            m.insert("analyticsTimeout", Exact("75000"));
            m.insert("kvTimeout", Exact("2500"));
            m.insert("connectTimeout", Exact("5000"));
            m.insert("dnsSrvEnabled", Exact("false"));
            m.insert("dcpConnectionName", Exact("dcp/core-io"));
            m.insert("retryStrategy", Exact("BestEffort"));
            m.insert("dcpConnectionBufferAckThreshold", Exact("0.2"));
            m.insert("dcpEnabled", Exact("false"));
            m.insert("dcpConnectionBufferSize", Exact("20971520"));
            m
        };
    }

impl ClientSettings {

    pub fn non_defaults(&self) -> HashMap<String, String> {
        let mut filtered = HashMap::new();
        for (key, val) in &self.inner {
            match CLIENT_SETTING_DEFAULTS.get(&key.as_ref()) {
                Some(t) => {
                    match t {
                        Exact(v) if v != val => {filtered.insert(key.clone(), v.to_string());},
                        Contains(v) if !val.contains(v) => {filtered.insert(key.clone(), v.to_string());},
                        _ => (),
                    }
                },
                None => {
                    filtered.insert(key.clone(), val.clone());
                }
            }
        }
        filtered
    }

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
                ClientSettings { inner: converted },
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
