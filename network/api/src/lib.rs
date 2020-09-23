// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::messages::PeerMessage;
use anyhow::*;
use async_trait::async_trait;
use network_rpc_core::RawRpcClient;
use starcoin_types::peer_info::RpcInfo;
use starcoin_types::system_events::NewHeadBlock;
use std::borrow::Cow;

pub mod messages;
mod peer_provider;

pub use libp2p::multiaddr::Multiaddr;
pub use peer_provider::PeerProvider;
pub use starcoin_types::peer_info::{PeerId, PeerInfo};

#[async_trait]
pub trait NetworkService:
    Send + Sync + Clone + Sized + std::marker::Unpin + RawRpcClient + PeerProvider
{
    async fn send_peer_message(
        &self,
        protocol_name: Cow<'static, [u8]>,
        peer_id: PeerId,
        msg: PeerMessage,
    ) -> Result<()>;
    async fn broadcast_new_head_block(
        &self,
        protocol_name: Cow<'static, [u8]>,
        event: NewHeadBlock,
    ) -> Result<()>;

    async fn register_rpc_proto(&self, rpc_info: RpcInfo) -> Result<()>;
}
