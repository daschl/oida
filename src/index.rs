use parse::LogLine;
use regex::Regex;
use chrono::NaiveDateTime;
use serde_cbor::{from_reader, to_writer};
use std::io::{Read, Write};
use std::collections::HashMap;

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
        let data = HashMap::new();

        // TODO: figure out how to parse this blob properly...
        // sslEnabled=false, sslKeystoreFile='null', sslKeystorePassword=false, sslKeystore=null, bootstrapHttpEnabled=true, 
        // bootstrapCarrierEnabled=true, bootstrapHttpDirectPort=8091, bootstrapHttpSslPort=18091, 
        // bootstrapCarrierDirectPort=11210, bootstrapCarrierSslPort=11207, ioPoolSize=3, computationPoolSize=3, 
        // responseBufferSize=16384, requestBufferSize=16384, kvServiceEndpoints=1, viewServiceEndpoints=1, 
        // queryServiceEndpoints=1, searchServiceEndpoints=1, ioPool=NioEventLoopGroup, coreScheduler=CoreScheduler, 
        // memcachedHashingStrategy=DefaultMemcachedHashingStrategy, eventBus=DefaultEventBus, 
        // packageNameAndVersion=couchbase-java-client/2.3.7 (git: 2.3.7, core: 1.3.7-dirty), dcpEnabled=false, 
        // retryStrategy=BestEffort, maxRequestLifetime=75000, 
        // retryDelay=ExponentialDelay{growBy 1.0 MICROSECONDS, powers of 2; lower=100, upper=100000}, 
        // reconnectDelay=ExponentialDelay{growBy 1.0 MILLISECONDS, powers of 2; lower=32, upper=4096}, 
        // observeIntervalDelay=ExponentialDelay{growBy 1.0 MICROSECONDS, powers of 2; lower=10, upper=100000}, 
        // keepAliveInterval=30000, autoreleaseAfter=2000, bufferPoolingEnabled=true, tcpNodelayEnabled=true, 
        // mutationTokensEnabled=false, socketConnectTimeout=1000, dcpConnectionBufferSize=20971520, 
        // dcpConnectionBufferAckThreshold=0.2, dcpConnectionName=dcp/core-io, callbacksOnIoPool=false, 
        // disconnectTimeout=25000, requestBufferWaitStrategy=com.couchbase.client.core.env.DefaultCoreEnvironment$2@5426e924, 
        // queryTimeout=75000, viewTimeout=75000, kvTimeout=2000, connectTimeout=3000, dnsSrvEnabled=true
        println!("{}", input);

        data
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) {
        to_writer(writer, &self).expect("Could not serialize into CBOR!")
    }
}
