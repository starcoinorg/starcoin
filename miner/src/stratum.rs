// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use sc_stratum::*;

use crate::miner::Miner;


struct StratumManager {
    miner: Miner
}

impl StratumManager {
    pub fn new(miner: Miner) -> Self {
        Self {
            miner
        }
    }
}

impl JobDispatcher for StratumManager {
    /*
    fn job(&self) -> Option<String> {
        //unimplemented!()
        //let job = self.miner.get_mint_job();
    }
    */
    fn submit(&self, payload: Vec<String>) -> Result<(), Error> {
        self.miner.submit(payload[0].clone().into_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::block::{Block, BlockTemplate, BlockHeader};

    #[test]
    fn test_stratum() {
        let addr = "127.0.0.1:19985".parse().unwrap();
        let miner = Miner::new();
        let mut miner_1 = miner.clone();
        let stratum = Stratum::start(&addr, Arc::new(StratumManager::new(miner)), None).unwrap();
        let block_template = {
            let block = Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test());
            BlockTemplate::from_block(block)
        };
        miner_1.set_mint_job(block_template);
        stratum.push_work_all(miner_1.get_mint_job());
        
        let request = r#"{"jsonrpc": "2.0", "method": "mining.subscribe", "params": [], "id": 1}"#;
        let resp = dummy_request(&addr, request);
        println!("{:?}", String::from_utf8(resp).unwrap());
        let submit_request = r#"{"jsonrpc": "2.0", "method": "mining.submit", "params": ["1","2","nihao"], "id": 1}"#;
        let resp = dummy_request(&addr, submit_request);
        println!("{:?}", String::from_utf8(resp).unwrap());
    }
}