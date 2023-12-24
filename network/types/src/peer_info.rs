// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
pub use network_p2p_types::multiaddr::Multiaddr;
pub use network_p2p_types::multihash::Multihash;
use network_p2p_types::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeader;
use starcoin_types::block::BlockNumber;
use starcoin_types::dag_block::AccumulatorInfo;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::U256;
use std::borrow::Cow;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub chain_info: ChainInfo,
    pub notif_protocols: Vec<Cow<'static, str>>,
    pub rpc_protocols: Vec<Cow<'static, str>>,
    pub version_string: Option<String>,
}

impl PeerInfo {
    pub fn new(
        peer_id: PeerId,
        chain_info: ChainInfo,
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
        version_string: Option<String>,
    ) -> Self {
        Self {
            peer_id,
            chain_info,
            notif_protocols,
            rpc_protocols,
            version_string,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn chain_info(&self) -> &ChainInfo {
        &self.chain_info
    }

    pub fn block_number(&self) -> BlockNumber {
        self.chain_info.head().number()
    }

    pub fn latest_header(&self) -> &BlockHeader {
        self.chain_info.head()
    }

    pub fn block_id(&self) -> HashValue {
        self.chain_info.head().id()
    }

    pub fn total_difficulty(&self) -> U256 {
        self.chain_info.total_difficulty()
    }

    pub fn update_chain_status(&mut self, chain_status: ChainStatus) {
        self.chain_info.update_status(chain_status)
    }

    /// This peer is support notification
    pub fn is_support_notification(&self) -> bool {
        !self.notif_protocols.is_empty()
    }

    pub fn is_support_notif_protocol(&self, protocol: Cow<'static, str>) -> bool {
        self.notif_protocols.contains(&protocol)
    }

    pub fn is_support_rpc(&self) -> bool {
        !self.rpc_protocols.is_empty()
    }

    pub fn is_support_rpc_protocol(&self, protocol: Cow<'static, str>) -> bool {
        self.rpc_protocols.contains(&protocol)
    }

    pub fn is_support_rpc_protocols(&self, protocols: &[Cow<'static, str>]) -> bool {
        for protocol in protocols {
            if !self.is_support_rpc_protocol(protocol.clone()) {
                return false;
            }
        }
        true
    }

    pub fn random() -> Self {
        Self {
            peer_id: PeerId::random(),
            chain_info: ChainInfo::random(),
            notif_protocols: vec![],
            rpc_protocols: vec![],
            version_string: None,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct RpcInfo {
    protocols: Vec<Cow<'static, str>>,
}

impl RpcInfo {
    pub const RPC_PROTOCOL_PREFIX: &'static str = "/starcoin/rpc/";

    pub fn is_empty(&self) -> bool {
        self.protocols.is_empty()
    }

    pub fn empty() -> Self {
        Self { protocols: vec![] }
    }
    pub fn new(mut paths: Vec<&'static str>) -> Self {
        paths.sort_unstable();
        paths.dedup();
        let protocols = paths
            .into_iter()
            .map(|path| {
                let protocol_name: Cow<'static, str> =
                    format!("{}{}", Self::RPC_PROTOCOL_PREFIX, path).into();
                protocol_name
            })
            .collect();
        Self { protocols }
    }

    pub fn into_protocols(self) -> Vec<Cow<'static, str>> {
        self.protocols
    }

    /// Get rpc path from protocol
    pub fn rpc_path(protocol: Cow<'static, str>) -> Result<String> {
        let path = protocol
            .strip_prefix(Self::RPC_PROTOCOL_PREFIX)
            .ok_or_else(|| {
                format_err!(
                    "Invalid rpc protocol {}, do not contains rpc protocol prefix",
                    protocol
                )
            })?;
        Ok(path.to_string())
    }
}

impl IntoIterator for RpcInfo {
    type Item = Cow<'static, str>;
    type IntoIter = std::vec::IntoIter<Cow<'static, str>>;

    fn into_iter(self) -> Self::IntoIter {
        self.protocols.into_iter()
    }
}
