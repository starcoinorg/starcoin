// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::*;
use bitflags::_core::time::Duration;
use futures::channel::mpsc::channel;
use futures::prelude::*;
use log::{debug, error};
use network_p2p::config::{RequestResponseConfig, TransportConfig};
use network_p2p::{
    identity, NetworkConfiguration, NetworkWorker, NodeKeyConfig, Params, ProtocolId, Secret,
};
use network_p2p_types::{is_memory_addr, ProtocolRequest};
use prometheus::default_registry;
use starcoin_config::NodeConfig;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::ServiceRef;
use starcoin_types::peer_info::RpcInfo;
use starcoin_types::startup_info::ChainInfo;
use std::borrow::Cow;

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
    let transport_config = if is_memory_addr(&node_config.network.listen()) {
        TransportConfig::MemoryOnly
    } else {
        TransportConfig::Normal {
            enable_mdns: !node_config.network.disable_mdns(),
            allow_private_ipv4: true,
            wasm_external_transport: None,
        }
    };
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
    let boot_nodes = node_config.network.seeds();
    let config = NetworkConfiguration {
        listen_addresses: vec![node_config.network.listen()],
        boot_nodes,
        node_key: {
            let secret = identity::ed25519::SecretKey::from_bytes(
                &mut node_config.network.network_keypair().0.to_bytes(),
            )
            .expect("decode network node key should success.");
            NodeKeyConfig::Ed25519(Secret::Input(secret))
        },
        notifications_protocols: protocols,
        request_response_protocols: rpc_protocols,
        transport: transport_config,
        node_name,
        client_version: starcoin_config::APP_NAME_WITH_VERSION.clone(),
        ..NetworkConfiguration::default()
    };
    // protocol id is chain/{chain_id}, `RegisteredProtocol` will append `/starcoin` prefix
    let protocol_id = ProtocolId::from(format!("chain/{}", chain_info.chain_id()).as_str());
    debug!("Init network worker with config: {:?}", config);
    let worker = NetworkWorker::new(Params::new(
        config,
        protocol_id,
        chain_info,
        //TODO use a custom registry for each instance.
        Some(default_registry().clone()),
    ))?;
    Ok(worker)
}
