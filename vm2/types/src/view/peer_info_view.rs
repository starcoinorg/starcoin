// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::chain_info_view::ChainInfoView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::PeerId;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PeerInfoView {
    pub peer_id: PeerId,
    pub chain_info: ChainInfoView,
    pub notif_protocols: String,
    pub rpc_protocols: String,
    pub version_string: Option<String>,
}

// TODO(BobOng): [dual-vm] put it into definaction file of PeerInfo
// impl From<PeerInfo> for PeerInfoView {
//     fn from(info: PeerInfo) -> Self {
//         Self {
//             peer_id: info.peer_id,
//             chain_info: info.chain_info.into(),
//             notif_protocols: info.notif_protocols.join(","),
//             rpc_protocols: info.rpc_protocols.join(","),
//             version_string: info.version_string,
//         }
//     }
// }
