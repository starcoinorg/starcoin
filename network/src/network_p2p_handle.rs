// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::anyhow;
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
        match ChainInfo::decode(peer_info) {
            Ok(other_chain_info) => {
                if self.chain_info.genesis_hash() == other_chain_info.genesis_hash() {
                    std::result::Result::Ok(())
                } else {
                    return Err((
                        "the genesis hash is different",
                        format!(
                            "the genesis hash from other peer is different, self: {}, remote: {}",
                            self.chain_info.genesis_hash(),
                            other_chain_info.genesis_hash()
                        ),
                    ));
                }
            }
            Err(error) => {
                return Err((
                    "failed to decode the generic data",
                    format!(
                        "failed to decode the generic data for the reason: {}",
                        error
                    ),
                ))
            }
        }
    }

    fn get_generic_data(&self) -> Result<Vec<u8>, anyhow::Error> {
        match self.chain_info.encode() {
            Ok(generic) => Ok(generic),
            Err(error) => Err(anyhow!(format!(
                "failed to encode chain info for the reason: {}",
                error
            ))),
        }
    }

    fn update_generic_data(&mut self, peer_info: &[u8]) -> Result<(), anyhow::Error> {
        match ChainInfo::decode(peer_info) {
            Ok(other_chain_info) => {
                self.chain_info = other_chain_info;
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
            Ok(status) => {
                self.chain_info.update_status(status);
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
}
