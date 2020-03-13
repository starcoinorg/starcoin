// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use sc_stratum::*;
use crate::miner::Miner;
use logger::prelude::*;

pub struct StratumManager {
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
    use jsonrpc_tcp_server::tokio::{io,runtime::Runtime,net::TcpStream,timer::{timeout,Timeout}};
    use jsonrpc_core::futures::{Future, future};
    use std::net::{SocketAddr, Shutdown};
    
    #[test]
    fn test_stratum() {
        /*
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
        let request = r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#;
        let response = String::from_utf8(dummy_request(&addr, request)).unwrap();
        println!("{:?}",response);
        */

        ::logger::init_for_test();
        let miner = Miner::new();
        let mut miner_1 = miner.clone();

        let block_template = {
            let block = Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test());
            BlockTemplate::from_block(block)
        };
        miner_1.set_mint_job(block_template);
        //stratum.push_work_all(miner_1.get_mint_job());
        
        let addr = "127.0.0.1:19995".parse().unwrap();
        let stratum = Stratum::start(&addr, Arc::new(StratumManager::new(miner)), None).unwrap();
        let mut auth_request =
            r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#
                .as_bytes()
                .to_vec();
        auth_request.extend(b"\n");
        let auth_response = "{\"jsonrpc\":\"2.0\",\"result\":true,\"id\":1}\n";
        let mut runtime = Runtime::new().expect("Tokio Runtime should be created with no errors");
        let read_buf0 = vec![0u8; auth_response.len()];
        let read_buf1 = Vec::with_capacity(2048);
        let stream = TcpStream::connect(&addr)
            .and_then(move |stream| {
                io::write_all(stream, auth_request)
            })
            .and_then(|(stream, _)| {
                io::read_exact(stream, read_buf0)
            })
            .map_err(|err| panic!("{:?}", err))
            .and_then(move |(stream, read_buf0)| {
                assert_eq!(String::from_utf8(read_buf0).unwrap(), auth_response);
                debug!(target: "stratum", "Received authorization confirmation");
                Timeout::new(future::ok(stream), ::std::time::Duration::from_millis(100))
            })
            .map_err(|err: timeout::Error<()>| panic!("Timeout: {:?}", err))
            .and_then(move |stream| {
                debug!(target: "stratum", "Pusing work to peers");
                stratum.push_work_all(r#"{ "00040008", "100500" }"#.to_owned())
                    .expect("Pushing work should produce no errors");
                Timeout::new(future::ok(stream), ::std::time::Duration::from_millis(100))
            })
            .map_err(|err: timeout::Error<()>| panic!("Timeout: {:?}", err))
            .and_then(|stream| {
                debug!(target: "stratum", "Ready to read work from server");
                stream.shutdown(Shutdown::Write).unwrap();
                io::read_to_end(stream, read_buf1)
            })
            .and_then(|(_, read_buf1)| {
                debug!(target: "stratum", "Received work from server");
                future::ok(read_buf1)
            });
        let response = String::from_utf8(
            runtime.block_on(stream).expect("Runtime should run with no errors")
        ).expect("Response should be utf-8");

        assert_eq!(
            "{ \"id\": 17, \"method\": \"mining.notify\", \"params\": { \"00040008\", \"100500\" } }\n",
            response);
    }
}