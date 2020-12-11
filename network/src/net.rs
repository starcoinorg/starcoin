// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{NetworkMessage, PeerEvent};
use anyhow::*;
use bitflags::_core::time::Duration;
use bytes::Bytes;
use config::NetworkConfig;
use config::NodeConfig;
use futures::channel::mpsc::channel;
use futures::{channel::mpsc, prelude::*};
use network_api::ReputationChange;
use network_p2p::config::{RequestResponseConfig, TransportConfig};
use network_p2p::{
    identity, Event, Multiaddr, NetworkConfiguration, NetworkService, NetworkWorker, NodeKeyConfig,
    Params, ProtocolId, Secret,
};
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::{is_memory_addr, PeerId, ProtocolRequest, RequestFailure};
use prometheus::{default_registry, Registry};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::ServiceRef;
use starcoin_types::peer_info::RpcInfo;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

const MAX_REQUEST_SIZE: u64 = 1024 * 1024;
const MAX_RESPONSE_SIZE: u64 = 1024 * 1024 * 64;
const REQUEST_BUFFER_SIZE: usize = 128;
pub const RPC_PROTOCOL_PREFIX: &str = "/starcoin/rpc/";

pub fn build_network_worker(
    node_config: &NodeConfig,
    chain_info: ChainInfo,
    protocols: Vec<Cow<'static, str>>,
    rpc_service: Option<(RpcInfo, ServiceRef<NetworkRpcService>)>,
) -> Result<NetworkWorker> {
    let node_name = node_config.node_name();
    let transport_config = if is_memory_addr(&node_config.network.listen) {
        TransportConfig::MemoryOnly
    } else {
        TransportConfig::Normal {
            //TODO support enable mdns by config.
            enable_mdns: false,
            allow_private_ipv4: false,
            wasm_external_transport: None,
        }
    };
    //let rpc_info: Vec<String> = starcoin_network_rpc_api::gen_client::get_rpc_info();
    //TODO define RequestResponseConfig by rpc api
    let rpc_protocols = match rpc_service {
        Some((rpc_info, rpc_service)) => rpc_info
            .into_iter()
            .map(|rpc_path| {
                //TODO define rpc path in rpc api, and add prefix.
                let protocol_name: Cow<'static, str> =
                    format!("{}{}", RPC_PROTOCOL_PREFIX, rpc_path.as_str()).into();
                let rpc_path_for_stream: Cow<'static, str> = rpc_path.into();
                let (sender, receiver) = channel(REQUEST_BUFFER_SIZE);
                let stream = receiver.map(move |request| ProtocolRequest {
                    protocol: rpc_path_for_stream.clone(),
                    request,
                });
                if let Err(e) = rpc_service.add_event_stream(stream) {
                    error!(
                        "Add request event stream for rpc {} fail: {:?}",
                        protocol_name, e
                    );
                }
                RequestResponseConfig {
                    name: protocol_name,
                    max_request_size: MAX_REQUEST_SIZE,
                    max_response_size: MAX_RESPONSE_SIZE,
                    request_timeout: Duration::from_secs(15),
                    inbound_queue: Some(sender),
                }
            })
            .collect::<Vec<_>>(),
        None => vec![],
    };
    let config = NetworkConfiguration {
        listen_addresses: vec![node_config.network.listen.clone()],
        boot_nodes: node_config.network.seeds.clone(),
        node_key: {
            let secret = identity::ed25519::SecretKey::from_bytes(
                &mut node_config.network.network_keypair().private_key.to_bytes(),
            )
            .expect("decode network node key should success.");
            NodeKeyConfig::Ed25519(Secret::Input(secret))
        },
        protocols,
        request_response_protocols: rpc_protocols,
        transport: transport_config,
        node_name: node_name.to_string(),
        client_version: config::APP_NAME_WITH_VERSION.clone(),
        ..NetworkConfiguration::default()
    };
    // protocol id is chain/{chain_id}, `RegisteredProtocol` will append `/starcoin` prefix
    let protocol_id = ProtocolId::from(format!("chain/{}", chain_info.chain_id()).as_str());

    let worker = NetworkWorker::new(Params::new(
        config,
        protocol_id,
        chain_info,
        //TODO use a custom registry for each instance.
        Some(default_registry().clone()),
    ))?;
    Ok(worker)
}
