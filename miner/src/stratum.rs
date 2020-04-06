// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::{MineCtx, Miner};
use anyhow::{format_err, Result};
use consensus::{Consensus, ConsensusHeader};
use sc_stratum::*;
use starcoin_wallet_api::WalletAccount;
use std::sync::Arc;
use traits::ChainReader;

use config::NodeConfig;
use logger::prelude::*;

use types::transaction::SignedUserTransaction;

pub struct StratumManager<H>
where
    H: ConsensusHeader + Sync + Send + 'static,
{
    miner: Miner<H>,
}

impl<H> StratumManager<H>
where
    H: ConsensusHeader + Sync + Send + 'static,
{
    pub fn new(miner: Miner<H>) -> Self {
        Self { miner }
    }
}

impl<H> JobDispatcher for StratumManager<H>
where
    H: ConsensusHeader + Sync + Send + 'static,
{
    fn submit(&self, payload: Vec<String>) -> Result<(), Error> {
        self.miner.submit(payload[0].clone().into_bytes());
        Ok(())
    }
}

pub fn mint<H, C>(
    stratum: Arc<Stratum>,
    mut miner: Miner<H>,
    config: Arc<NodeConfig>,
    miner_account: WalletAccount,
    txns: Vec<SignedUserTransaction>,
    chain: &dyn ChainReader,
) -> Result<()>
where
    H: ConsensusHeader,
    C: Consensus,
{
    let block_template = chain.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        None,
        C::calculate_next_difficulty(config, chain),
        txns,
    )?;
    miner.set_mint_job(MineCtx::new(block_template));
    let job = miner.get_mint_job();
    info!("Push job to worker{:?}", job);
    stratum
        .push_work_all(job)
        .map_err(|e| format_err!("stratum push failed:{:?}", e))?;
    Ok(())
}
