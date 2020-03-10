// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use sc_stratum::*;
use crate::miner::Miner;


struct StratumManager {
    miner: Miner
}

impl JobDispatcher for StratumManager {
    fn initial(&self) -> Option<String> {
        unimplemented!()
        //let job = self.miner.get_mint_job();
        //return job;
    }
    fn job(&self) -> Option<String> {
        unimplemented!()
        //let job = self.miner.get_mint_job();
    }
    fn submit(&self, payload: Vec<String>) -> Result<(), Error> {
        unimplemented!()
        //self.miner.submit(payload);
        //Ok(())
    }
}
