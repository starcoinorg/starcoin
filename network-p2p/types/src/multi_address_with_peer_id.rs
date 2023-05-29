use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use libp2p::core::{multiaddr, Multiaddr, PeerId};

use crate::ParseErr;

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
