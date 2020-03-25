use crate::ProtocolId;
use bitflags::_core::hash::Hash;
use bytes::Bytes;
use futures::channel::oneshot;
use futures::future::{BoxFuture, FutureExt};
use futures::stream::{FuturesUnordered, StreamExt};
use futures::{AsyncRead, AsyncWrite};
use libp2p::{
    core::{
        upgrade::{read_one, write_one, OutboundUpgrade},
        upgrade::{InboundUpgrade, Negotiated, ReadOneError, UpgradeInfo},
        ConnectedPoint, Multiaddr, PeerId,
    },
    swarm::{
        NegotiatedSubstream, NetworkBehaviour, NetworkBehaviourAction, OneShotHandler,
        PollParameters, SubstreamProtocol,
    },
};
use nohash_hasher::IntMap;
use peerset::ReputationChange;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::task::{Context, Poll};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{io, iter};
use void::Void;

/// Remote request timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
/// Default request retry count.
const RETRY_COUNT: usize = 1;
/// Reputation change for a peer when a request timed out.
pub(crate) const TIMEOUT_REPUTATION_CHANGE: i32 = -(1 << 8);

#[derive(Debug, Clone)]
pub struct Config {
    max_data_size: usize,
    max_pending_requests: usize,
    inactivity_timeout: Duration,
    request_timeout: Duration,
    protocol: Bytes,
}

impl Config {
    /// Create a fresh configuration with the following options:
    ///
    /// - max. data size = 1 MiB
    /// - max. pending requests = 128
    /// - inactivity timeout = 15s
    /// - request timeout = 15s
    pub fn new(id: &ProtocolId) -> Self {
        let mut c = Config {
            max_data_size: 1024 * 1024,
            max_pending_requests: 128,
            inactivity_timeout: Duration::from_secs(15),
            request_timeout: Duration::from_secs(15),
            protocol: Bytes::new(),
        };
        c.set_protocol(id);
        c
    }

    /// Limit the max. length of incoming request bytes.
    pub fn set_max_data_size(&mut self, v: usize) -> &mut Self {
        self.max_data_size = v;
        self
    }

    /// Limit the max. number of pending requests.
    pub fn set_max_pending_requests(&mut self, v: usize) -> &mut Self {
        self.max_pending_requests = v;
        self
    }

    /// Limit the max. duration the connection may remain inactive before closing it.
    pub fn set_inactivity_timeout(&mut self, v: Duration) -> &mut Self {
        self.inactivity_timeout = v;
        self
    }

    /// Limit the max. request duration.
    pub fn set_request_timeout(&mut self, v: Duration) -> &mut Self {
        self.request_timeout = v;
        self
    }

    /// Set protocol to use for upgrade negotiation.
    pub fn set_protocol(&mut self, id: &ProtocolId) -> &mut Self {
        let mut v = Vec::new();
        v.extend_from_slice(b"/");
        v.extend_from_slice(id.as_bytes());
        v.extend_from_slice(b"/rpc/1");
        self.protocol = v.into();
        self
    }
}

/// Possible errors while handling light clients.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// There are currently too many pending request.
    #[error("too many pending requests")]
    TooManyRequests,
    /// The response type does not correspond to the issued request.
    #[error("unexpected response")]
    UnexpectedResponse,
    /// A bad request has been received.
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("Remote data fetch has been cancelled")]
    RemoteFetchCancelled,
    /// Remote fetch has been failed.
    #[error("Remote data fetch has been failed")]
    RemoteFetchFailed,
}

#[derive(Debug)]
pub enum Event<T> {
    /// Incoming request from remote and substream to use for the response.
    Request(Request, T),
    /// Incoming response from remote.
    Response(Response),
}

/// Augments a light client request with metadata.
#[derive(Debug)]
struct RequestWrapper {
    /// Time when this value was created.
    timestamp: Instant,
    /// Remaining retries.
    retries: usize,
    /// The actual request.
    request: Vec<u8>,
    /// Peer information, e.g. `PeerId`.
    peer: PeerId,

    id: u128,

    sender: oneshot::Sender<Result<Vec<u8>, Error>>,
}

/// message from peer
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    id: u128,
    data: Vec<u8>,
}

/// message from peer
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    id: u128,
    data: Vec<u8>,
}

fn send_reply(response: Result<Vec<u8>, Error>, request: RequestWrapper) {
    let _ = request.sender.send(response);
}

pub struct RpcHandler {
    /// This behaviour's configuration.
    config: Config,
    /// Futures sending back response to remote clients.
    responses: FuturesUnordered<BoxFuture<'static, ()>>,
    /// Pending (local) requests.
    pending_requests: VecDeque<RequestWrapper>,
    /// Requests on their way to remote peers.
    outstanding: HashMap<u128, RequestWrapper>,
    /// (Local) Request ID counter
    next_request_id: u128,
    /// Handle to use for reporting misbehaviour of peers.
    peerset: peerset::PeersetHandle,

    peers: HashMap<PeerId, PeerInfo>,
}

/// Information we have about some peer.
#[derive(Debug)]
struct PeerInfo {
    address: Multiaddr,
    status: PeerStatus,
}

/// A peer is either idle or busy processing a request from us.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PeerStatus {
    /// The peer is available.
    Idle,
    /// We wait for the peer to return us a response for the given request ID.
    BusyWith(u128),
}

/// Substream upgrade protocol.
///
/// Reads incoming requests from remote.
#[derive(Debug, Clone)]
pub struct InboundProtocol {
    /// The max. request length in bytes.
    max_data_size: usize,
    /// The protocol to use for upgrade negotiation.
    protocol: Bytes,
}

/// Substream upgrade protocol.
///
/// Sends a request to remote and awaits the response.
#[derive(Debug, Clone)]
pub struct OutboundProtocol {
    /// The serialized protobuf request.
    request: Vec<u8>,
    /// The max. request length in bytes.
    max_data_size: usize,
    /// The protocol to use for upgrade negotiation.
    protocol: Bytes,
}

impl UpgradeInfo for OutboundProtocol {
    type Info = Bytes;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(self.protocol.clone())
    }
}

impl<T> OutboundUpgrade<T> for OutboundProtocol
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Output = Event<T>;
    type Error = ReadOneError;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, mut s: T, _: Self::Info) -> Self::Future {
        let future = async move {
            write_one(&mut s, &self.request).await?;
            let vec = read_one(&mut s, self.max_data_size).await?;
            let response = Response::decode(&vec).unwrap();
            Ok(Event::Response(response))
        };
        future.boxed()
    }
}

impl UpgradeInfo for InboundProtocol {
    type Info = Bytes;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(self.protocol.clone())
    }
}

impl<T> InboundUpgrade<T> for InboundProtocol
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Output = Event<T>;
    type Error = ReadOneError;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, mut s: T, _: Self::Info) -> Self::Future {
        let future = async move {
            let vec = read_one(&mut s, self.max_data_size).await?;
            let request = Request::decode(&vec).unwrap();
            Ok(Event::Request(request, s))
        };
        future.boxed()
    }
}

impl RpcHandler {
    pub fn new(cfg: Config, peerset: peerset::PeersetHandle) -> Self {
        Self {
            config: cfg,
            responses: FuturesUnordered::new(),
            pending_requests: VecDeque::new(),
            outstanding: HashMap::default(),
            next_request_id: 1,
            peerset,
            peers: HashMap::new(),
        }
    }

    fn remove_peer(&mut self, peer: &PeerId) {
        if let Some(id) = self
            .outstanding
            .iter()
            .find(|(_, rw)| &rw.peer == peer)
            .map(|(k, _)| *k)
        {
            let rw = self
                .outstanding
                .remove(&id)
                .expect("key belongs to entry in this map");
            let rw = RequestWrapper {
                id: get_id(),
                timestamp: rw.timestamp,
                retries: rw.retries,
                request: rw.request,
                peer: peer.clone(), // need to find another peer
                sender: rw.sender,
            };
            self.pending_requests.push_back(rw);
        }
        self.peers.remove(peer);
    }

    /// Issue a new light client request.
    pub fn request(
        &mut self,
        peer: PeerId,
        data: Vec<u8>,
    ) -> Result<oneshot::Receiver<Result<Vec<u8>, Error>>, Error> {
        if self.pending_requests.len() >= self.config.max_pending_requests {
            return Err(Error::TooManyRequests);
        }
        let (tx, rx) = oneshot::channel();
        let rw = RequestWrapper {
            id: get_id(),
            timestamp: Instant::now(),
            retries: 3,
            request: data,
            peer,
            sender: tx,
        };
        self.pending_requests.push_back(rw);
        Ok(rx)
    }
}

impl NetworkBehaviour for RpcHandler {
    type ProtocolsHandler =
        OneShotHandler<InboundProtocol, OutboundProtocol, Event<NegotiatedSubstream>>;
    type OutEvent = Void;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        let p = InboundProtocol {
            max_data_size: self.config.max_data_size,
            protocol: self.config.protocol.clone(),
        };
        OneShotHandler::new(SubstreamProtocol::new(p), self.config.inactivity_timeout)
    }

    fn addresses_of_peer(&mut self, peer: &PeerId) -> Vec<Multiaddr> {
        self.peers
            .get(peer)
            .map(|info| vec![info.address.clone()])
            .unwrap_or_default()
    }

    fn inject_connected(&mut self, peer: PeerId, info: ConnectedPoint) {
        let peer_address = match info {
            ConnectedPoint::Listener { send_back_addr, .. } => send_back_addr,
            ConnectedPoint::Dialer { address } => address,
        };

        log::trace!("peer {} connected with address {}", peer, peer_address);

        let info = PeerInfo {
            address: peer_address,
            status: PeerStatus::Idle,
        };

        self.peers.insert(peer, info);
    }

    fn inject_disconnected(&mut self, peer: &PeerId, _: ConnectedPoint) {
        log::trace!("peer {} disconnected", peer);
        self.remove_peer(peer)
    }

    fn inject_node_event(&mut self, peer: PeerId, event: Event<NegotiatedSubstream>) {
        match event {
            // An incoming request from remote has been received.
            Event::Request(request, stream) => {
                log::trace!("incoming request from {}", peer);
            }
            // A response to one of our own requests has been received.
            Event::Response(response) => {
                let id = response.id;
                if let Some(rw) = self.outstanding.remove(&id) {
                    send_reply(Ok(response.data), rw);
                }
            }
        }
    }

    fn poll(
        &mut self,
        cx: &mut Context,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<OutboundProtocol, Void>> {
        while let Poll::Ready(Some(_)) = self.responses.poll_next_unpin(cx) {}

        // If we have a pending request to send, try to find an available peer and send it.
        let now = Instant::now();
        while let Some(mut request) = self.pending_requests.pop_front() {
            if now > request.timestamp + self.config.request_timeout {
                if request.retries == 0 {
                    send_reply(Err(Error::RemoteFetchFailed), request);
                    continue;
                }
                request.timestamp = Instant::now();
                request.retries -= 1
            }

            let id = get_id();
            log::trace!("sending request {} to peer {}", id, request.peer);
            let sq = Request {
                id,
                data: request.request.to_vec(),
            };
            let protocol = OutboundProtocol {
                request: sq.encode().unwrap(),
                max_data_size: self.config.max_data_size,
                protocol: self.config.protocol.clone(),
            };
            self.peers
                .get_mut(&request.peer)
                .map(|info| info.status = PeerStatus::BusyWith(id));
            let rw = RequestWrapper {
                id: get_id(),
                timestamp: request.timestamp,
                retries: request.retries,
                request: request.request,
                peer: request.peer.clone(),
                sender: request.sender,
            };
            self.outstanding.insert(id, rw);
            return Poll::Ready(NetworkBehaviourAction::SendEvent {
                peer_id: request.peer,
                event: protocol,
            });
        }

        // Look for ongoing requests that have timed out.
        let mut expired = Vec::new();
        for (id, rw) in &self.outstanding {
            if now > rw.timestamp + self.config.request_timeout {
                log::debug!("request {} timed out", id);
                expired.push(*id)
            }
        }
        for id in expired {
            if let Some(rw) = self.outstanding.remove(&id) {
                self.remove_peer(&rw.peer);
                self.peerset.report_peer(
                    rw.peer.clone(),
                    ReputationChange::new(TIMEOUT_REPUTATION_CHANGE, "light request timeout"),
                );
                if rw.retries == 0 {
                    send_reply(Err(Error::RemoteFetchFailed), rw);
                    continue;
                }
                let rw = RequestWrapper {
                    id: rw.id,
                    timestamp: Instant::now(),
                    retries: rw.retries - 1,
                    request: rw.request,
                    peer: rw.peer,
                    sender: rw.sender,
                };
                self.pending_requests.push_back(rw)
            }
        }
        Poll::Pending
    }
}

fn get_id() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_nanos()
}
