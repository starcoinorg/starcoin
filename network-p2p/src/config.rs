// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Libp2p network configuration.

use crypto::HashValue;
use libp2p::{
    core::Multiaddr,
    identity::{ed25519, Keypair},
    multiaddr::Protocol,
    wasm_ext, {multiaddr, PeerId},
};

use std::borrow::Cow;
use std::fmt;
use std::{convert::TryFrom, str::FromStr};
use std::{
    error::Error,
    fs,
    io::{self, Write},
    iter,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};
use types::peer_info::PeerInfo;
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtocolId(smallvec::SmallVec<[u8; 6]>);

impl<'a> From<&'a [u8]> for ProtocolId {
    fn from(bytes: &'a [u8]) -> ProtocolId {
        ProtocolId(bytes.into())
    }
}

impl ProtocolId {
    /// Exposes the `ProtocolId` as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Network initialization parameters.
pub struct Params {
    /// Network layer configuration.
    pub network_config: NetworkConfiguration,

    /// Name of the protocol to use on the wire. Should be different for each chain.
    pub protocol_id: ProtocolId,
}

impl Params {
    pub fn new(network_config: NetworkConfiguration, protocol_id: ProtocolId) -> Self {
        Self {
            network_config,
            protocol_id,
        }
    }
}

/// Network service configuration.
#[derive(Clone, Debug)]
pub struct NetworkConfiguration {
    /// Directory path to store general network configuration. None means nothing will be saved.
    pub config_path: Option<String>,
    /// Directory path to store network-specific configuration. None means nothing will be saved.
    pub net_config_path: Option<String>,
    /// Multiaddresses to listen for incoming connections.
    pub listen_addresses: Vec<Multiaddr>,
    /// Multiaddresses to advertise. Detected automatically if empty.
    pub public_addresses: Vec<Multiaddr>,
    /// List of initial node addresses
    pub boot_nodes: Vec<Multiaddr>,
    /// The node key configuration, which determines the node's network identity keypair.
    pub node_key: NodeKeyConfig,
    /// Maximum allowed number of incoming connections.
    pub in_peers: u32,
    /// Number of outgoing connections we're trying to maintain.
    pub out_peers: u32,
    /// List of reserved node addresses.
    pub reserved_nodes: Vec<MultiaddrWithPeerId>,
    /// The non-reserved peer mode.
    pub non_reserved_mode: NonReservedPeerMode,
    /// Client identifier. Sent over the wire for debugging purposes.
    pub client_version: String,
    /// Name of the node. Sent over the wire for debugging purposes.
    pub node_name: String,

    pub transport: TransportConfig,

    pub genesis_hash: HashValue,

    pub self_info: PeerInfo,

    pub disable_seed: bool,

    pub protocols: Vec<Cow<'static, [u8]>>,
}

/// Configuration for the transport layer.
#[derive(Clone, Debug)]
pub enum TransportConfig {
    /// Normal transport mode.
    Normal {
        /// If true, the network will use mDNS to discover other libp2p nodes on the local network
        /// and connect to them if they support the same chain.
        enable_mdns: bool,

        /// If true, allow connecting to private IPv4 addresses (as defined in
        /// [RFC1918](https://tools.ietf.org/html/rfc1918)), unless the address has been passed in
        /// [`NetworkConfiguration::reserved_nodes`] or [`NetworkConfiguration::boot_nodes`].
        allow_private_ipv4: bool,

        /// Optional external implementation of a libp2p transport. Used in WASM contexts where we
        /// need some binding between the networking provided by the operating system or environment
        /// and libp2p.
        ///
        /// This parameter exists whatever the target platform is, but it is expected to be set to
        /// `Some` only when compiling for WASM.
        wasm_external_transport: Option<wasm_ext::ExtTransport>,
        /// Use flow control for yamux streams if set to true.
        use_yamux_flow_control: bool,
    },

    /// Only allow connections within the same process.
    /// Only addresses of the form `/memory/...` will be supported.
    MemoryOnly,
}

impl Default for NetworkConfiguration {
    fn default() -> Self {
        NetworkConfiguration {
            config_path: None,
            net_config_path: None,
            listen_addresses: Vec::new(),
            public_addresses: Vec::new(),
            boot_nodes: Vec::new(),
            node_key: NodeKeyConfig::Ed25519(Secret::New),
            in_peers: 25,
            out_peers: 75,
            reserved_nodes: Vec::new(),
            non_reserved_mode: NonReservedPeerMode::Accept,
            client_version: "unknown".into(),
            node_name: "unknown".into(),
            transport: TransportConfig::Normal {
                enable_mdns: false,
                allow_private_ipv4: false,
                wasm_external_transport: None,
                use_yamux_flow_control: false,
            },
            genesis_hash: HashValue::default(),
            self_info: PeerInfo::default(),
            disable_seed: false,
            protocols: vec![],
        }
    }
}

impl NetworkConfiguration {
    /// Create a new instance of default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new default configuration for localhost-only connection with random port (useful for
    /// testing)
    pub fn new_local() -> NetworkConfiguration {
        let mut config = NetworkConfiguration::new();
        config.listen_addresses = vec![iter::once(Protocol::Ip4(Ipv4Addr::new(127, 0, 0, 1)))
            .chain(iter::once(Protocol::Tcp(0)))
            .collect()];
        config
    }
}

/// The policy for connections to non-reserved peers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NonReservedPeerMode {
    /// Accept them. This is the default.
    Accept,
    /// Deny them.
    Deny,
}

impl NonReservedPeerMode {
    /// Attempt to parse the peer mode from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "accept" => Some(NonReservedPeerMode::Accept),
            "deny" => Some(NonReservedPeerMode::Deny),
            _ => None,
        }
    }
}

/// The configuration of a node's secret key, describing the type of key
/// and how it is obtained. A node's identity keypair is the result of
/// the evaluation of the node key configuration.
#[derive(Clone, Debug)]
pub enum NodeKeyConfig {
    /// A Ed25519 secret key configuration.
    Ed25519(Secret<ed25519::SecretKey>),
}

/// The configuration options for obtaining a secret key `K`.
#[derive(Clone)]
pub enum Secret<K> {
    /// Use the given secret key `K`.
    Input(K),
    /// Read the secret key from a file. If the file does not exist,
    /// it is created with a newly generated secret key `K`. The format
    /// of the file is determined by `K`:
    ///
    ///   * `ed25519::SecretKey`: An unencoded 32 bytes Ed25519 secret key.
    File(PathBuf),
    /// Always generate a new secret key `K`.
    New,
}

impl<K> fmt::Debug for Secret<K> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Secret::Input(_) => f.debug_tuple("Secret::Input").finish(),
            Secret::File(path) => f.debug_tuple("Secret::File").field(path).finish(),
            Secret::New => f.debug_tuple("Secret::New").finish(),
        }
    }
}

impl NodeKeyConfig {
    /// Evaluate a `NodeKeyConfig` to obtain an identity `Keypair`:
    ///
    ///  * If the secret is configured as input, the corresponding keypair is returned.
    ///
    ///  * If the secret is configured as a file, it is read from that file, if it exists.
    ///    Otherwise a new secret is generated and stored. In either case, the
    ///    keypair obtained from the secret is returned.
    ///
    ///  * If the secret is configured to be new, it is generated and the corresponding
    ///    keypair is returned.
    pub fn into_keypair(self) -> io::Result<Keypair> {
        use NodeKeyConfig::*;
        match self {
            Ed25519(Secret::New) => Ok(Keypair::generate_ed25519()),

            Ed25519(Secret::Input(k)) => Ok(Keypair::Ed25519(k.into())),

            Ed25519(Secret::File(f)) => get_secret(
                f,
                |mut b| ed25519::SecretKey::from_bytes(&mut b),
                ed25519::SecretKey::generate,
                |b| b.as_ref().to_vec(),
            )
            .map(ed25519::Keypair::from)
            .map(Keypair::Ed25519),
        }
    }
}

/// Load a secret key from a file, if it exists, or generate a
/// new secret key and write it to that file. In either case,
/// the secret key is returned.
fn get_secret<P, F, G, E, W, K>(file: P, parse: F, generate: G, serialize: W) -> io::Result<K>
where
    P: AsRef<Path>,
    F: for<'r> FnOnce(&'r mut [u8]) -> Result<K, E>,
    G: FnOnce() -> K,
    E: Error + Send + Sync + 'static,
    W: Fn(&K) -> Vec<u8>,
{
    std::fs::read(&file)
        .and_then(|mut sk_bytes| {
            parse(&mut sk_bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
        .or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                file.as_ref().parent().map_or(Ok(()), fs::create_dir_all)?;
                let sk = generate();
                let mut sk_vec = serialize(&sk);
                write_secret_file(file, &sk_vec)?;
                sk_vec.zeroize();
                Ok(sk)
            } else {
                Err(e)
            }
        })
}

/// Write secret bytes to a file.
fn write_secret_file<P>(path: P, sk_bytes: &[u8]) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut file = open_secret_file(&path)?;
    file.write_all(sk_bytes)
}

/// Opens a file containing a secret key in write mode.
#[cfg(unix)]
fn open_secret_file<P>(path: P) -> io::Result<fs::File>
where
    P: AsRef<Path>,
{
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

/// Opens a file containing a secret key in write mode.
#[cfg(not(unix))]
fn open_secret_file<P>(path: P) -> Result<fs::File, io::Error>
where
    P: AsRef<Path>,
{
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
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

/// Address of a node, including its identity.
///
/// This struct represents a decoded version of a multiaddress that ends with `/p2p/<peerid>`.
///
/// # Example
///
/// ```
/// # use sc_network::{Multiaddr, PeerId, config::MultiaddrWithPeerId};
/// let addr: MultiaddrWithPeerId =
/// 	"/ip4/198.51.100.19/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV".parse().unwrap();
/// assert_eq!(addr.peer_id.to_base58(), "QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV");
/// assert_eq!(addr.multiaddr.to_string(), "/ip4/198.51.100.19/tcp/30333");
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MultiaddrWithPeerId {
    /// Address of the node.
    pub multiaddr: Multiaddr,
    /// Its identity.
    pub peer_id: PeerId,
}

impl MultiaddrWithPeerId {
    /// Concatenates the multiaddress and peer ID into one multiaddress containing both.
    pub fn concat(&self) -> Multiaddr {
        let proto = multiaddr::Protocol::P2p(From::from(self.peer_id.clone()));
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

/// Parses a string address and splits it into Multiaddress and PeerId, if
/// valid.
///
/// # Example
///
/// ```
/// # use sc_network::{Multiaddr, PeerId, config::parse_str_addr};
/// let (peer_id, addr) = parse_str_addr(
/// 	"/ip4/198.51.100.19/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV"
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
            if !matches!(key.algorithm(), multiaddr::multihash::Code::Identity) {
                // (note: this is the "person bowing" emoji)
                log::warn!(
                    "🙇 You are using the peer ID {}. This peer ID uses a legacy, deprecated \
					representation that will no longer be supported in the future. \
					Please refresh it by performing a RPC query to the appropriate node, \
					by looking at its logs, or by using `subkey inspect-node-key` on its \
					private key.",
                    bs58::encode(key.as_bytes()).into_string()
                );
            }

            PeerId::from_multihash(key).map_err(|_| ParseErr::InvalidPeerId)?
        }
        _ => return Err(ParseErr::PeerIdMissing),
    };

    Ok((who, addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tempdir_with_prefix(prefix: &str) -> TempDir {
        tempfile::Builder::new().prefix(prefix).tempdir().unwrap()
    }

    fn secret_bytes(kp: &Keypair) -> Vec<u8> {
        match kp {
            Keypair::Ed25519(p) => p.secret().as_ref().to_vec(),
            Keypair::Secp256k1(p) => p.secret().to_bytes().to_vec(),
            _ => panic!("Unexpected keypair."),
        }
    }

    #[test]
    fn test_secret_file() {
        let tmp = tempdir_with_prefix("x");
        std::fs::remove_dir(tmp.path()).unwrap(); // should be recreated
        let file = tmp.path().join("x");
        let kp1 = NodeKeyConfig::Ed25519(Secret::File(file.clone()))
            .into_keypair()
            .unwrap();
        let kp2 = NodeKeyConfig::Ed25519(Secret::File(file.clone()))
            .into_keypair()
            .unwrap();
        assert!(file.is_file() && secret_bytes(&kp1) == secret_bytes(&kp2))
    }

    #[test]
    fn test_secret_input() {
        let sk = ed25519::SecretKey::generate();
        let kp1 = NodeKeyConfig::Ed25519(Secret::Input(sk.clone()))
            .into_keypair()
            .unwrap();
        let kp2 = NodeKeyConfig::Ed25519(Secret::Input(sk))
            .into_keypair()
            .unwrap();
        assert!(secret_bytes(&kp1) == secret_bytes(&kp2));
    }

    #[test]
    fn test_secret_new() {
        let kp1 = NodeKeyConfig::Ed25519(Secret::New).into_keypair().unwrap();
        let kp2 = NodeKeyConfig::Ed25519(Secret::New).into_keypair().unwrap();
        assert!(secret_bytes(&kp1) != secret_bytes(&kp2));
    }
}
