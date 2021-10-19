// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::*;
use bitflags::_core::time::Duration;
use futures::channel::mpsc::channel;
use futures::prelude::*;
use log::{debug, error, info};
use network_api::PeerInfo;
use network_p2p::config::{RequestResponseConfig, TransportConfig};
use network_p2p::{
    identity, NetworkConfiguration, NetworkWorker, NodeKeyConfig, Params, ProtocolId, Secret,
};
use network_p2p_types::{is_memory_addr, ProtocolRequest};
use starcoin_config::NetworkConfig;
use starcoin_metrics::Registry;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::ServiceRef;
use starcoin_types::peer_info::RpcInfo;
use starcoin_types::startup_info::ChainInfo;
use std::borrow::Cow;

const MAX_REQUEST_SIZE: u64 = 1024 * 1024;
const MAX_RESPONSE_SIZE: u64 = 1024 * 1024 * 64;
const REQUEST_BUFFER_SIZE: usize = 128;
pub const RPC_PROTOCOL_PREFIX: &str = RpcInfo::RPC_PROTOCOL_PREFIX;

pub fn build_network_worker(
    network_config: &NetworkConfig,
    chain_info: ChainInfo,
    protocols: Vec<Cow<'static, str>>,
    rpc_service: Option<(RpcInfo, ServiceRef<NetworkRpcService>)>,
    metrics_registry: Option<Registry>,
) -> Result<(PeerInfo, NetworkWorker)> {
    let node_name = network_config.node_name();
    let discover_local = network_config.discover_local();
    let transport_config = if is_memory_addr(&network_config.listen()) {
        TransportConfig::MemoryOnly
    } else {
        TransportConfig::Normal {
            enable_mdns: discover_local,
            allow_private_ipv4: true,
            wasm_external_transport: None,
        }
    };
    //TODO define RequestResponseConfig by rpc api
    let rpc_protocols = match rpc_service {
        Some((rpc_info, rpc_service)) => rpc_info
            .into_protocols()
            .into_iter()
            .map(move |rpc_protocol| {
                let (sender, receiver) = channel(REQUEST_BUFFER_SIZE);
                let protocol_for_stream = rpc_protocol.clone();
                let stream = receiver.map(move |request| ProtocolRequest {
                    protocol: protocol_for_stream.clone(),
                    request,
                });
                if let Err(e) = rpc_service.add_event_stream(stream) {
                    error!(
                        "Add request event stream for rpc {} fail: {:?}",
                        rpc_protocol, e
                    );
                }
                RequestResponseConfig {
                    name: rpc_protocol,
                    max_request_size: MAX_REQUEST_SIZE,
                    max_response_size: MAX_RESPONSE_SIZE,
                    request_timeout: Duration::from_secs(30),
                    inbound_queue: Some(sender),
                }
            })
            .collect::<Vec<_>>(),
        None => vec![],
    };
    let allow_non_globals_in_dht = discover_local;
    let boot_nodes = network_config.seeds();

    info!("Final bootstrap seeds: {:?}", boot_nodes);
    let self_info = PeerInfo::new(
        network_config.self_peer_id(),
        chain_info.clone(),
        protocols.to_vec(),
        rpc_protocols
            .iter()
            .map(|config| config.name.clone())
            .collect(),
    );
    let config = NetworkConfiguration {
        listen_addresses: vec![network_config.listen()],
        boot_nodes,
        node_key: {
            let secret = identity::ed25519::SecretKey::from_bytes(
                &mut network_config.network_keypair().0.to_bytes(),
            )
            .expect("decode network node key should success.");
            NodeKeyConfig::Ed25519(Secret::Input(secret))
        },
        in_peers: network_config.max_incoming_peers(),
        out_peers: network_config.max_outgoing_peers(),
        notifications_protocols: protocols,
        request_response_protocols: rpc_protocols,
        transport: transport_config,
        node_name,
        client_version: starcoin_config::APP_NAME_WITH_VERSION.clone(),
        allow_non_globals_in_dht,
        ..NetworkConfiguration::default()
    };
    // protocol id is chain/{chain_id}, `RegisteredProtocol` will append `/starcoin` prefix
    let protocol_id = ProtocolId::from(format!("chain/{}", chain_info.chain_id()).as_str());
    debug!("Init network worker with config: {:?}", config);

    let worker = NetworkWorker::new(Params::new(
        config,
        protocol_id,
        chain_info,
        metrics_registry,
    ))?;

    Ok((self_info, worker))
}
