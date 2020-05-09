// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::{MineCtx, Miner};
use config::NodeConfig;
use logger::prelude::*;
use sc_stratum::*;
use starcoin_wallet_api::WalletAccount;
use std::sync::Arc;
use traits::ChainReader;
use traits::Consensus;
use types::transaction::SignedUserTransaction;

pub struct StratumManager<C>
where
    C: Consensus + Sync + Send + 'static,
{
    miner: Miner<C>,
}

impl<C> StratumManager<C>
where
    C: Consensus + Sync + Send + 'static,
{
    pub fn new(miner: Miner<C>) -> Self {
        Self { miner }
    }
}

impl<C> JobDispatcher for StratumManager<C>
where
    C: Consensus + Sync + Send + 'static,
{
    fn submit(&self, payload: Vec<String>) -> Result<(), Error> {
        //todo:: error handle
        let _ = self.miner.submit(payload[0].clone());
        Ok(())
    }
}

pub fn mint<C>(
    stratum: Arc<Stratum>,
    mut miner: Miner<C>,
    config: Arc<NodeConfig>,
    miner_account: WalletAccount,
    txns: Vec<SignedUserTransaction>,
    chain: &dyn ChainReader,
) -> anyhow::Result<()>
where
    C: Consensus,
{
    let block_template = chain.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        None,
        txns,
    )?;
    let difficulty = C::calculate_next_difficulty(config, chain);
    miner.set_mint_job(MineCtx::new(block_template, difficulty));
    let job = miner.get_mint_job();
    info!("Push job to worker {}", job);
    if let Err(e) = stratum.push_work_all(job) {
        error!("Stratum push failed:{:?}", e);
    }
    Ok(())
}
