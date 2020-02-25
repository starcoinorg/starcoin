// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use anyhow::Result;
use anyhow::Result;
use network_libp2p::PeerId;
use std::{
    convert::TryFrom,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use types::account_address::AccountAddress;

pub fn convert_peer_id_to_account_address(peer_id: &PeerId) -> Result<AccountAddress> {
    let peer_id_bytes = &peer_id.as_bytes()[2..];
    AccountAddress::try_from(peer_id_bytes)
}

pub fn convert_account_address_to_peer_id(
    address: AccountAddress,
) -> std::result::Result<PeerId, Vec<u8>> {
    let mut peer_id_vec = address.to_vec();
    peer_id_vec.insert(0, 32);
    peer_id_vec.insert(0, 22);
    PeerId::from_bytes(peer_id_vec)
}

pub fn convert_boot_nodes(boot_nodes: Vec<String>) -> Vec<String> {
    boot_nodes
        .iter()
        .map(|x| {
            let dx = x.rfind("/").expect("Failed to parse boot nodes");

            let account_address = &x[dx + 1..];
            let addr = &x[..dx];
            let peer_id = convert_account_address_to_peer_id(
                AccountAddress::from_str(account_address).expect("Failed to parse account address"),
            )
            .expect("Failed to parse account address");
            format!("{:}/{:}", addr, peer_id).to_string()
        })
        .clone()
        .collect()
}

pub fn get_unix_ts() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_nanos()
}
