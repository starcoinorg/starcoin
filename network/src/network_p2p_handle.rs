// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use bcs_ext::BCSCodec;
use network_p2p_types::business_layer_handle::BusinessLayerHandle;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};

pub struct Networkp2pHandle {
    chain_info: ChainInfo,
}

impl Networkp2pHandle {
    pub fn new(chain_info: ChainInfo) -> Self {
        Networkp2pHandle { chain_info }
    }
}

impl BusinessLayerHandle for Networkp2pHandle {
    fn handshake(&self, peer_info: &[u8]) -> Result<(), (&'static str, String)> {
        let other_chain_info = ChainInfo::decode(peer_info).unwrap();
        if self.chain_info.genesis_hash() == other_chain_info.genesis_hash() {
            return std::result::Result::Ok(());
        }
        return Err((
            "the genesis hash is different",
            format!(
                "the genesis hash from other peer is different, self: {}, remote: {}",
                self.chain_info.genesis_hash(),
                other_chain_info.genesis_hash()
            ),
        ));
    }

    fn get_generic_data(&self) -> Result<Vec<u8>, anyhow::Error> {
        Ok(self.chain_info.encode().unwrap())
    }

    fn update_generic_data(&mut self, peer_info: &[u8]) -> Result<(), anyhow::Error> {
        self.chain_info = ChainInfo::decode(peer_info).unwrap();
        Ok(())
    }

    fn update_status(&mut self, peer_status: &[u8]) -> Result<(), anyhow::Error> {
        self.chain_info
            .update_status(ChainStatus::decode(peer_status).unwrap());
        Ok(())
    }
}
