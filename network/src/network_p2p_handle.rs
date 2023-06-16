use std::{borrow::Cow, error};

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{anyhow, Error, Ok};
use bcs_ext::BCSCodec;
use log::{debug, Level};
use log::{error, log};
use network_p2p::{
    business_layer_handle::BusinessLayerHandle,
    protocol::{generic_proto::NotificationsSink, rep, CustomMessageOutcome},
    PeerId,
};
use sc_peerset::{ReputationChange, SetId};
use serde::{Deserialize, Serialize};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};

/// Current protocol version.
pub(crate) const CURRENT_VERSION: u32 = 5;
/// Lowest version we support
pub(crate) const MIN_VERSION: u32 = 3;

/// Status sent on connection.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Protocol version.
    pub version: u32,
    /// Minimum supported version.
    pub min_supported_version: u32,
    /// Tell other peer which notification protocols we support.
    pub notif_protocols: Vec<Cow<'static, str>>,
    /// Tell other peer which rpc api we support.
    pub rpc_protocols: Vec<Cow<'static, str>>,
    /// the generic data related to the peer
    pub info: ChainInfo,
}

pub struct Networkp2pHandle {
    status: Status,
}

impl Networkp2pHandle {
    pub fn new(chain_info: ChainInfo) -> Self {
        let status = Status {
            version: CURRENT_VERSION,
            min_supported_version: MIN_VERSION,
            notif_protocols: [].to_vec(),
            rpc_protocols: [].to_vec(),
            info: chain_info,
        };
        Networkp2pHandle { status }
    }
}

impl Networkp2pHandle {
    fn inner_handshake(
        &self,
        who: PeerId,
        set_id: SetId,
        protocol_name: Cow<'static, str>,
        status: Status,
        notifications_sink: NotificationsSink,
    ) -> Result<CustomMessageOutcome, ReputationChange> {
        debug!(target: "network-p2p", "New peer {} {:?}", who, status);
        if status.info.genesis_hash() != self.status.info.genesis_hash() {
            error!(
                target: "network-p2p",
                "Bootnode with peer id `{}` is on a different chain (our genesis: {} theirs: {})",
                who,
                self.status.info.genesis_hash(),
                status.info.genesis_hash(),
            );
            return Err(rep::GENESIS_MISMATCH);
        }
        if status.version < MIN_VERSION || CURRENT_VERSION < status.min_supported_version {
            log!(
                target: "network-p2p",
                Level::Warn,
                "Peer {:?} using unsupported protocol version {}", who, status.version
            );
            return Err(rep::BAD_PROTOCOL);
        }
        debug!(target: "network-p2p", "Connected {}", who);
        let result_generic_data = status.info.encode();
        match result_generic_data {
            std::result::Result::Ok(generic_data) => {
                return std::result::Result::Ok(CustomMessageOutcome::NotificationStreamOpened {
                    remote: who,
                    protocol: protocol_name,
                    notifications_sink,
                    generic_data,
                    notif_protocols: status.notif_protocols.to_vec(),
                    rpc_protocols: status.rpc_protocols.to_vec(),
                });
            }
            Err(error) => {
                return Err(rep::FAILED_TO_ENCODE);
            }
        }
    }
}

impl BusinessLayerHandle for Networkp2pHandle {
    fn handshake(
        &self,
        peer_id: PeerId,
        set_id: SetId,
        protocol_name: Cow<'static, str>,
        received_handshake: Vec<u8>,
        notifications_sink: NotificationsSink,
    ) -> Result<CustomMessageOutcome, ReputationChange> {
        match Status::decode(&received_handshake[..]) {
            std::result::Result::Ok(status) => {
                return self.inner_handshake(
                    peer_id,
                    set_id,
                    protocol_name,
                    status,
                    notifications_sink,
                );
            }
            Err(err) => {
                error!(target: "network-p2p", "Couldn't decode handshake packet sent by {}: {:?}: {}", peer_id, hex::encode(received_handshake), err);
                return Err(rep::BAD_MESSAGE);
            }
        }
    }

    fn get_generic_data(&self) -> Result<Vec<u8>, anyhow::Error> {
        self.status.encode()
    }

    fn update_generic_data(&mut self, peer_info: &[u8]) -> Result<(), anyhow::Error> {
        match ChainInfo::decode(peer_info) {
            std::result::Result::Ok(other_chain_info) => {
                self.status.info = other_chain_info;
                Ok(())
            }
            Err(error) => {
                return Err(anyhow!(
                    "failed to decode the generic data for the reason: {}",
                    error
                ))
            }
        }
    }

    fn update_status(&mut self, peer_status: &[u8]) -> Result<(), anyhow::Error> {
        match ChainStatus::decode(peer_status) {
            std::result::Result::Ok(status) => {
                self.status.info.update_status(status);
                Ok(())
            }
            Err(error) => {
                return Err(anyhow!(
                    "failed to decode the generic data for the reason: {}",
                    error
                ))
            }
        }
    }

    fn build_handshake_msg(
        &mut self,
        notif_protocols: Vec<std::borrow::Cow<'static, str>>,
        rpc_protocols: Vec<std::borrow::Cow<'static, str>>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        self.status.notif_protocols = notif_protocols;
        self.status.rpc_protocols = rpc_protocols;
        self.status.encode()
    }
}
