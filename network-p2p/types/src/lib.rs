// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "arm",
    target_arch = "arrch64"
))]
use libp2p::futures::channel::oneshot;
#[cfg(target_arch = "mips")]
use libp2p_in_mips::futures::channel::oneshot;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

pub mod network_state;

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "arm",
    target_arch = "arrch64"
))]
pub use libp2p::core::{identity, multiaddr, Multiaddr, PeerId, PublicKey};
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "arm",
    target_arch = "arrch64"
))]
pub use libp2p::request_response::{InboundFailure, OutboundFailure};
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "arm",
    target_arch = "arrch64"
))]
pub use libp2p::{build_multiaddr, multihash};

#[cfg(target_arch = "mips")]
pub use libp2p_in_mips::core::{identity, multiaddr, Multiaddr, PeerId, PublicKey};
#[cfg(target_arch = "mips")]
pub use libp2p_in_mips::request_response::{InboundFailure, OutboundFailure};
#[cfg(target_arch = "mips")]
pub use libp2p_in_mips::{build_multiaddr, multihash};

pub use sc_peerset::{ReputationChange, BANNED_THRESHOLD};

/// Parses a string address and splits it into Multiaddress and PeerId, if
/// valid.
///
/// # Example
///
/// ```
/// # use network_p2p_types::{Multiaddr, PeerId, parse_str_addr};
/// let (peer_id, addr) = parse_str_addr(
///    "/ip4/198.51.100.19/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV"
/// ).unwrap();
/// assert_eq!(peer_id, "QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV".parse::<PeerId>().unwrap());
/// assert_eq!(addr, "/ip4/198.51.100.19/tcp/30333".parse::<Multiaddr>().unwrap());
/// ```
///
pub fn parse_str_addr(addr_str: &str) -> Result<(PeerId, Multiaddr), ParseErr> {
    let addr: Multiaddr = addr_str.parse()?;
    parse_addr(addr)
}

/// Splits a Multiaddress into a Multiaddress and PeerId.
pub fn parse_addr(mut addr: Multiaddr) -> Result<(PeerId, Multiaddr), ParseErr> {
    let who = match addr.pop() {
        Some(multiaddr::Protocol::P2p(key)) => {
            PeerId::from_multihash(key).map_err(|_| ParseErr::InvalidPeerId)?
        }
        _ => return Err(ParseErr::PeerIdMissing),
    };

    Ok((who, addr))
}

/// Build memory protocol Multiaddr by port
pub fn memory_addr(port: u64) -> Multiaddr {
    build_multiaddr!(Memory(port))
}

/// Generate a random memory protocol Multiaddr
pub fn random_memory_addr() -> Multiaddr {
    memory_addr(rand::random::<u64>())
}

/// Check the address is a memory protocol Multiaddr.
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "arm",
    target_arch = "arrch64"
))]
pub fn is_memory_addr(addr: &Multiaddr) -> bool {
    addr.iter()
        .any(|protocol| matches!(protocol, libp2p::core::multiaddr::Protocol::Memory(_)))
}

#[cfg(target_arch = "mips")]
pub fn is_memory_addr(addr: &Multiaddr) -> bool {
    addr.iter().any(|protocol| {
        matches!(
            protocol,
            libp2p_in_mips::core::multiaddr::Protocol::Memory(_)
        )
    })
}

/// Address of a node, including its identity.
///
/// This struct represents a decoded version of a multiaddress that ends with `/p2p/<peerid>`.
///
/// # Example
///
/// ```
/// # use network_p2p_types::{Multiaddr, PeerId, MultiaddrWithPeerId};
/// let addr: MultiaddrWithPeerId =
///     "/ip4/198.51.100.19/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV".parse().unwrap();
/// assert_eq!(addr.peer_id.to_base58(), "QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV");
/// assert_eq!(addr.multiaddr.to_string(), "/ip4/198.51.100.19/tcp/30333");
/// ```
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct MultiaddrWithPeerId {
    /// Address of the node.
    pub multiaddr: Multiaddr,
    /// Its identity.
    pub peer_id: PeerId,
}

impl MultiaddrWithPeerId {
    pub fn new(multiaddr: Multiaddr, peer_id: PeerId) -> Self {
        Self { multiaddr, peer_id }
    }

    /// Concatenates the multiaddress and peer ID into one multiaddress containing both.
    pub fn concat(&self) -> Multiaddr {
        let proto = multiaddr::Protocol::P2p(From::from(self.peer_id));
        self.multiaddr.clone().with(proto)
    }
}

impl fmt::Display for MultiaddrWithPeerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.concat(), f)
    }
}

impl FromStr for MultiaddrWithPeerId {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (peer_id, multiaddr) = parse_str_addr(s)?;
        Ok(MultiaddrWithPeerId { peer_id, multiaddr })
    }
}

impl From<MultiaddrWithPeerId> for String {
    fn from(ma: MultiaddrWithPeerId) -> String {
        format!("{}", ma)
    }
}

impl TryFrom<String> for MultiaddrWithPeerId {
    type Error = ParseErr;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        string.parse()
    }
}

#[allow(clippy::from_over_into)]
impl Into<Multiaddr> for MultiaddrWithPeerId {
    fn into(self) -> Multiaddr {
        self.concat()
    }
}

/// Error that can be generated by `parse_str_addr`.
#[derive(Debug)]
pub enum ParseErr {
    /// Error while parsing the multiaddress.
    MultiaddrParse(multiaddr::Error),
    /// Multihash of the peer ID is invalid.
    InvalidPeerId,
    /// The peer ID is missing from the address.
    PeerIdMissing,
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErr::MultiaddrParse(err) => write!(f, "{}", err),
            ParseErr::InvalidPeerId => write!(f, "Peer id at the end of the address is invalid"),
            ParseErr::PeerIdMissing => write!(f, "Peer id is missing from the address"),
        }
    }
}

impl std::error::Error for ParseErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseErr::MultiaddrParse(err) => Some(err),
            ParseErr::InvalidPeerId => None,
            ParseErr::PeerIdMissing => None,
        }
    }
}

impl From<multiaddr::Error> for ParseErr {
    fn from(err: multiaddr::Error) -> ParseErr {
        ParseErr::MultiaddrParse(err)
    }
}

/// Error in a request.
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum RequestFailure {
    /// We are not currently connected to the requested peer.
    NotConnected,
    /// Given protocol hasn't been registered.
    UnknownProtocol,
    /// Remote has closed the substream before answering, thereby signaling that it considers the
    /// request as valid, but refused to answer it.
    Refused,
    /// The remote replied, but the local node is no longer interested in the response.
    Obsolete,
    /// Problem on the network.
    #[display(fmt = "Problem on the network: {:?}", _0)]
    Network(#[error(ignore)] OutboundFailure),
}

/// Error when processing a request sent by a remote.
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ResponseFailure {
    /// Problem on the network.
    #[display(fmt = "Problem on the network: {:?}", _0)]
    Network(#[error(ignore)] InboundFailure),
}

/// Response for an incoming request to be send by a request protocol handler.
#[derive(Debug)]
pub struct OutgoingResponse {
    /// The payload of the response.
    ///
    /// `Err(())` if none is available e.g. due an error while handling the request.
    pub result: Result<Vec<u8>, ()>,
    /// Reputation changes accrued while handling the request. To be applied to the reputation of
    /// the peer sending the request.
    pub reputation_changes: Vec<ReputationChange>,
}

/// A single request received by a peer on a request-response protocol.
#[derive(Debug)]
pub struct IncomingRequest {
    /// Who sent the request.
    pub peer: PeerId,

    /// Request sent by the remote. Will always be smaller than
    /// [`ProtocolConfig::max_request_size`].
    pub payload: Vec<u8>,

    /// Channel to send back the response.
    ///
    /// There are two ways to indicate that handling the request failed:
    ///
    /// 1. Drop `pending_response` and thus not changing the reputation of the peer.
    ///
    /// 2. Sending an `Err(())` via `pending_response`, optionally including reputation changes for
    /// the given peer.
    pub pending_response: oneshot::Sender<OutgoingResponse>,
}

#[derive(Debug)]
pub struct ProtocolRequest {
    pub protocol: Cow<'static, str>,
    pub request: IncomingRequest,
}

/// When sending a request, what to do on a disconnected recipient.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IfDisconnected {
    /// Try to connect to the peer.
    TryConnect,
    /// Just fail if the destination is not yet connected.
    ImmediateError,
}

/// Convenience functions for `IfDisconnected`.
impl IfDisconnected {
    /// Shall we connect to a disconnected peer?
    pub fn should_connect(self) -> bool {
        match self {
            Self::TryConnect => true,
            Self::ImmediateError => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_address() {
        let addr = random_memory_addr();
        assert!(is_memory_addr(&addr));
    }
}
