// Copyright 2015-2019 Parity Technologies (UK) Ltd.

//! Stratum protocol implementation for parity ethereum/bitcoin clients

extern crate ethereum_types;
extern crate jsonrpc_core;
extern crate jsonrpc_tcp_server;
extern crate keccak_hash as hash;
extern crate parking_lot;

#[macro_use]
extern crate log;

#[cfg(test)]
extern crate tokio;
#[cfg(test)]
extern crate tokio_io;

use jsonrpc_core::futures::{future, Future};
use jsonrpc_tcp_server::tokio::{io, net::TcpStream, runtime::Runtime};
use std::net::{Shutdown, SocketAddr};

mod traits;

pub use traits::{Error, JobDispatcher, PushWorkHandler, ServiceConfiguration};

use jsonrpc_core::{to_value, Compatibility, IoDelegate, MetaIoHandler, Metadata, Params, Value};
use jsonrpc_tcp_server::{
    Dispatcher, MetaExtractor, PushMessageError, RequestContext, Server as JsonRpcServer,
    ServerBuilder as JsonRpcServerBuilder,
};
use std::sync::Arc;

use ethereum_types::H256;
use hash::keccak;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};

type RpcResult = Result<jsonrpc_core::Value, jsonrpc_core::Error>;

const NOTIFY_COUNTER_INITIAL: u32 = 16;

/// Container which owns rpc server and stratum implementation
pub struct Stratum {
    /// RPC server
    ///
    /// It is an `Option` so it can be easily closed and released during `drop` phase
    rpc_server: Option<JsonRpcServer>,
    /// stratum protocol implementation
    ///
    /// It is owned by a container and rpc server
    implementation: Arc<StratumImpl>,
    /// Message dispatcher (tcp/ip service)
    ///
    /// Used to push messages to peers
    tcp_dispatcher: Dispatcher,
}

impl Stratum {
    pub fn start(
        addr: &SocketAddr,
        dispatcher: Arc<dyn JobDispatcher>,
        secret: Option<H256>,
    ) -> Result<Arc<Stratum>, Error> {
        let implementation = Arc::new(StratumImpl {
            subscribers: RwLock::default(),
            job_que: RwLock::default(),
            dispatcher,
            workers: Arc::new(RwLock::default()),
            secret,
            notify_counter: RwLock::new(NOTIFY_COUNTER_INITIAL),
        });

        let mut delegate = IoDelegate::<StratumImpl, SocketMetadata>::new(implementation.clone());
        delegate.add_method_with_meta("mining.subscribe", StratumImpl::subscribe);
        delegate.add_method_with_meta("mining.authorize", StratumImpl::authorize);
        delegate.add_method_with_meta("mining.submit", StratumImpl::submit);
        let mut handler = MetaIoHandler::<SocketMetadata>::with_compatibility(Compatibility::Both);
        handler.extend_with(delegate);

        let server_builder = JsonRpcServerBuilder::new(handler);
        let tcp_dispatcher = server_builder.dispatcher();
        let server_builder =
            server_builder.session_meta_extractor(PeerMetaExtractor::new(tcp_dispatcher.clone()));
        let server = server_builder.start(addr)?;

        let stratum = Arc::new(Stratum {
            rpc_server: Some(server),
            implementation,
            tcp_dispatcher,
        });

        Ok(stratum)
    }
}

impl PushWorkHandler for Stratum {
    fn push_work_all(&self, payload: String) -> Result<(), Error> {
        self.implementation
            .push_work_all(payload, &self.tcp_dispatcher)
    }

    fn push_work(&self, payloads: Vec<String>) -> Result<(), Error> {
        self.implementation
            .push_work(payloads, &self.tcp_dispatcher)
    }
}

impl Drop for Stratum {
    fn drop(&mut self) {
        // shut down rpc server
        if let Some(server) = self.rpc_server.take() {
            server.close()
        };
    }
}

struct StratumImpl {
    /// Subscribed clients
    subscribers: RwLock<Vec<SocketAddr>>,
    /// List of workers supposed to receive job update
    job_que: RwLock<HashSet<SocketAddr>>,
    /// Payload manager
    dispatcher: Arc<dyn JobDispatcher>,
    /// Authorized workers (socket - worker_id)
    workers: Arc<RwLock<HashMap<SocketAddr, String>>>,
    /// Secret if any
    secret: Option<H256>,
    /// Dispatch notify counter
    notify_counter: RwLock<u32>,
}

impl StratumImpl {
    /// rpc method `mining.subscribe`
    fn subscribe(&self, _params: Params, meta: SocketMetadata) -> RpcResult {
        use std::str::FromStr;

        self.subscribers.write().push(*meta.addr());
        self.job_que.write().insert(*meta.addr());
        trace!(target: "stratum", "Subscription request from {:?}", meta.addr());

        Ok(match self.dispatcher.initial() {
            Some(initial) => match jsonrpc_core::Value::from_str(&initial) {
                Ok(val) => Ok(val),
                Err(e) => {
                    warn!(target: "stratum", "Invalid payload: '{}' ({:?})", &initial, e);
                    to_value(&[0u8; 0])
                }
            },
            None => to_value(&[0u8; 0]),
        }
        .expect("Empty slices are serializable; qed"))
    }

    /// rpc method `mining.authorize`
    fn authorize(&self, params: Params, meta: SocketMetadata) -> RpcResult {
        params
            .parse::<(String, String)>()
            .map(|(worker_id, secret)| {
                if let Some(valid_secret) = self.secret {
                    let hash = keccak(secret);
                    if hash != valid_secret {
                        return to_value(&false);
                    }
                }
                trace!(target: "stratum", "New worker #{} registered", worker_id);
                self.workers.write().insert(*meta.addr(), worker_id);
                to_value(true)
            })
            .map(|v| v.expect("Only true/false is returned and it's always serializable; qed"))
    }

    /// rpc method `mining.submit`
    fn submit(&self, params: Params, _meta: SocketMetadata) -> RpcResult {
        Ok(match params {
            Params::Array(vals) => {
                // first two elements are service messages (worker_id & job_id)
                match self.dispatcher.submit(
                    vals.iter()
                        .skip(2)
                        .filter_map(|val| match *val {
                            Value::String(ref s) => Some(s.to_owned()),
                            _ => None,
                        })
                        .collect::<Vec<String>>(),
                ) {
                    Ok(()) => {
                        // Do not update peers in submit
                        //self.update_peers(&meta.tcp_dispatcher.expect("tcp_dispatcher is always initialized; qed"));
                        to_value(true)
                    }
                    Err(submit_err) => {
                        warn!("Error while submitting share: {:?}", submit_err);
                        to_value(false)
                    }
                }
            }
            _ => {
                trace!(target: "stratum", "Invalid submit work format {:?}", params);
                to_value(false)
            }
        }
        .expect("Only true/false is returned and it's always serializable; qed"))
    }

    /// Helper method
    fn _update_peers(&self, tcp_dispatcher: &Dispatcher) {
        if let Some(job) = self.dispatcher.job() {
            if let Err(e) = self.push_work_all(job, tcp_dispatcher) {
                warn!("Failed to update some of the peers: {:?}", e);
            }
        }
    }

    fn push_work_all(&self, payload: String, tcp_dispatcher: &Dispatcher) -> Result<(), Error> {
        let workers = self.workers.read();
        if workers.is_empty() {
            return Err(Error::NoWorkers);
        }
        let hup_peers = {
            let next_request_id = {
                let mut counter = self.notify_counter.write();
                if *counter == ::std::u32::MAX {
                    *counter = NOTIFY_COUNTER_INITIAL;
                } else {
                    *counter += 1
                }
                *counter
            };

            let mut hup_peers = HashSet::with_capacity(0); // most of the cases won't be needed, hence avoid allocation
            let workers_msg = format!(
                "{{ \"id\": {}, \"method\": \"mining.notify\", \"params\": {} }}",
                next_request_id, payload
            );
            trace!(target: "stratum", "pushing work for {} workers (payload: '{}')", workers.len(), &workers_msg);
            for (ref addr, _) in workers.iter() {
                trace!(target: "stratum", "pusing work to {}", addr);
                match tcp_dispatcher.push_message(addr, workers_msg.clone()) {
                    Err(PushMessageError::NoSuchPeer) => {
                        trace!(target: "stratum", "Worker no longer connected: {}", addr);
                        hup_peers.insert(**addr);
                    }
                    Err(e) => {
                        warn!(target: "stratum", "Unexpected transport error: {:?}", e);
                    }
                    Ok(_) => {}
                }
            }
            hup_peers
        };

        if !hup_peers.is_empty() {
            let mut workers = self.workers.write();
            for hup_peer in hup_peers {
                workers.remove(&hup_peer);
            }
        }

        Ok(())
    }

    fn push_work(&self, payloads: Vec<String>, tcp_dispatcher: &Dispatcher) -> Result<(), Error> {
        if !payloads.len() > 0 {
            return Err(Error::NoWork);
        }
        let workers = self.workers.read();
        let addrs = workers.keys().collect::<Vec<&SocketAddr>>();
        if !workers.len() > 0 {
            return Err(Error::NoWorkers);
        }
        let mut que = payloads;
        let mut addr_index = 0;
        while !que.is_empty() {
            let next_worker = addrs[addr_index];
            let mut next_payload = que.drain(0..1);
            tcp_dispatcher.push_message(
                next_worker,
                next_payload
                    .next()
                    .expect("drained successfully of 0..1, so 0-th element should exist"),
            )?;
            addr_index += 1;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct SocketMetadata {
    addr: SocketAddr,
    // with the new version of jsonrpc-core, SocketMetadata
    // won't have to implement default, so this field will not
    // have to be an Option
    tcp_dispatcher: Option<Dispatcher>,
}

impl Default for SocketMetadata {
    fn default() -> Self {
        SocketMetadata {
            addr: "0.0.0.0:0".parse().unwrap(),
            tcp_dispatcher: None,
        }
    }
}

impl SocketMetadata {
    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }
}

impl Metadata for SocketMetadata {}

pub struct PeerMetaExtractor {
    tcp_dispatcher: Dispatcher,
}

impl PeerMetaExtractor {
    fn new(tcp_dispatcher: Dispatcher) -> Self {
        PeerMetaExtractor { tcp_dispatcher }
    }
}

impl MetaExtractor<SocketMetadata> for PeerMetaExtractor {
    fn extract(&self, context: &RequestContext) -> SocketMetadata {
        SocketMetadata {
            addr: context.peer_addr,
            tcp_dispatcher: Some(self.tcp_dispatcher.clone()),
        }
    }
}

pub fn dummy_request(addr: &SocketAddr, data: &str) -> Vec<u8> {
    let mut runtime = Runtime::new().expect("Tokio Runtime should be created with no errors");

    let mut data_vec = data.as_bytes().to_vec();
    data_vec.extend(b"\n");

    let stream = TcpStream::connect(addr)
        .and_then(move |stream| io::write_all(stream, data_vec))
        .and_then(|(stream, _)| {
            stream.shutdown(Shutdown::Write).unwrap();
            io::read_to_end(stream, Vec::with_capacity(2048))
        })
        .and_then(|(_stream, read_buf)| future::ok(read_buf));
    runtime
        .block_on(stream)
        .expect("Runtime should run with no errors")
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_tcp_server::tokio::timer::{timeout, Timeout};

    pub struct VoidManager;

    impl JobDispatcher for VoidManager {
        fn submit(&self, _payload: Vec<String>) -> Result<(), Error> {
            Ok(())
        }
    }

    #[test]
    fn can_be_started() {
        let stratum = Stratum::start(
            &format!("127.0.0.1:{}", starcoin_config::get_random_available_port())
                .parse()
                .unwrap(),
            Arc::new(VoidManager),
            None,
        );
        assert!(stratum.is_ok());
    }

    #[test]
    fn records_subscriber() {
        starcoin_logger::init_for_test();

        let addr = format!("127.0.0.1:{}", starcoin_config::get_random_available_port())
            .parse()
            .unwrap();
        let stratum = Stratum::start(&addr, Arc::new(VoidManager), None).unwrap();
        let request = r#"{"jsonrpc": "2.0", "method": "mining.subscribe", "params": [], "id": 1}"#;
        dummy_request(&addr, request);
        assert_eq!(1, stratum.implementation.subscribers.read().len());
    }

    struct DummyManager {
        initial_payload: String,
    }

    impl DummyManager {
        fn new() -> Arc<DummyManager> {
            Arc::new(Self::build())
        }

        fn build() -> DummyManager {
            DummyManager {
                initial_payload: r#"[ "dummy payload" ]"#.to_owned(),
            }
        }

        fn of_initial(mut self, new_initial: &str) -> DummyManager {
            self.initial_payload = new_initial.to_owned();
            self
        }
    }

    impl JobDispatcher for DummyManager {
        fn initial(&self) -> Option<String> {
            Some(self.initial_payload.clone())
        }

        fn submit(&self, _payload: Vec<String>) -> Result<(), Error> {
            Ok(())
        }
    }

    fn terminated_str(origin: &'static str) -> String {
        let mut s = String::new();
        s.push_str(origin);
        s.push_str("\n");
        s
    }

    #[test]
    fn receives_initial_payload() {
        let addr = format!("127.0.0.1:{}", starcoin_config::get_random_available_port())
            .parse()
            .unwrap();
        let _stratum = Stratum::start(&addr, DummyManager::new(), None)
            .expect("There should be no error starting stratum");
        let request = r#"{"jsonrpc": "2.0", "method": "mining.subscribe", "params": [], "id": 2}"#;

        let response = String::from_utf8(dummy_request(&addr, request)).unwrap();

        assert_eq!(
            terminated_str(r#"{"jsonrpc":"2.0","result":["dummy payload"],"id":2}"#),
            response
        );
    }

    #[test]
    fn can_authorize() {
        let addr = format!("127.0.0.1:{}", starcoin_config::get_random_available_port())
            .parse()
            .unwrap();
        let stratum = Stratum::start(
            &addr,
            Arc::new(DummyManager::build().of_initial(r#"["dummy autorize payload"]"#)),
            None,
        )
        .expect("There should be no error starting stratum");

        let request = r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#;
        let response = String::from_utf8(dummy_request(&addr, request)).unwrap();

        assert_eq!(
            terminated_str(r#"{"jsonrpc":"2.0","result":true,"id":1}"#),
            response
        );
        assert_eq!(1, stratum.implementation.workers.read().len());
    }

    #[test]
    fn can_push_work() {
        starcoin_logger::init_for_test();

        let addr = format!("127.0.0.1:{}", starcoin_config::get_random_available_port())
            .parse()
            .unwrap();
        let stratum = Stratum::start(
            &addr,
            Arc::new(DummyManager::build().of_initial(r#"["dummy autorize payload"]"#)),
            None,
        )
        .expect("There should be no error starting stratum");

        let mut auth_request =
            r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#
                .as_bytes()
                .to_vec();
        auth_request.extend(b"\n");

        let auth_response = "{\"jsonrpc\":\"2.0\",\"result\":true,\"id\":1}\n";

        let mut runtime = Runtime::new().expect("Tokio Runtime should be created with no errors");
        let read_buf0 = vec![0u8; auth_response.len()];
        let read_buf1 = Vec::with_capacity(2048);
        let stream = TcpStream::connect(&addr)
            .and_then(move |stream| io::write_all(stream, auth_request))
            .and_then(|(stream, _)| io::read_exact(stream, read_buf0))
            .map_err(|err| panic!("{:?}", err))
            .and_then(move |(stream, read_buf0)| {
                assert_eq!(String::from_utf8(read_buf0).unwrap(), auth_response);
                trace!(target: "stratum", "Received authorization confirmation");
                Timeout::new(future::ok(stream), ::std::time::Duration::from_millis(100))
            })
            .map_err(|err: timeout::Error<()>| panic!("Timeout: {:?}", err))
            .and_then(move |stream| {
                trace!(target: "stratum", "Pusing work to peers");
                stratum
                    .push_work_all(r#"{ "00040008", "100500" }"#.to_owned())
                    .expect("Pushing work should produce no errors");
                Timeout::new(future::ok(stream), ::std::time::Duration::from_millis(100))
            })
            .map_err(|err: timeout::Error<()>| panic!("Timeout: {:?}", err))
            .and_then(|stream| {
                trace!(target: "stratum", "Ready to read work from server");
                stream.shutdown(Shutdown::Write).unwrap();
                io::read_to_end(stream, read_buf1)
            })
            .and_then(|(_, read_buf1)| {
                trace!(target: "stratum", "Received work from server");
                future::ok(read_buf1)
            });
        let response = String::from_utf8(
            runtime
                .block_on(stream)
                .expect("Runtime should run with no errors"),
        )
        .expect("Response should be utf-8");

        assert_eq!(
            "{ \"id\": 17, \"method\": \"mining.notify\", \"params\": { \"00040008\", \"100500\" } }\n",
            response);
    }
}
