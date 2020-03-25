use anyhow::Result;
use argon2::{self, Config};
use async_std::{io::BufReader, net::TcpStream, prelude::*, task};
use byteorder::{LittleEndian, WriteBytesExt};
use futures::channel::mpsc;
use jsonrpc_core::{MethodCall, Params};
use logger::prelude::*;
use rand::Rng;
use serde_json;
use std::{net::SocketAddr, sync::Arc};
use types::{H256, U256};

pub fn verify(header: &[u8], nonce: u64, difficulty: U256) -> bool {
    let pow_header = MinerClient::set_header_nonce(header, nonce);
    let pow_hash = MinerClient::calculate_hash(&pow_header);
    let hash_u256: U256 = pow_hash.into();
    if hash_u256 <= difficulty {
        return true;
    }
    return false;
}

pub struct MinerClient {}

impl MinerClient {
    pub fn set_header_nonce(header: &[u8], nonce: u64) -> Vec<u8> {
        let len = header.len();
        let mut header = header.to_owned();
        header.truncate(len - 8);
        let _ = WriteBytesExt::write_u64::<LittleEndian>(&mut header, nonce);
        header
    }

    pub fn calculate_hash(header: &[u8]) -> H256 {
        let config = Config::default();
        let output = argon2::hash_raw(header, header, &config).unwrap();
        let h_256: H256 = output.as_slice().into();
        h_256
    }

    pub fn solve(difficulty: U256, header: &[u8]) -> u64 {
        let mut nonce = MinerClient::generate_nonce();
        loop {
            let pow_hash =
                MinerClient::calculate_hash(&MinerClient::set_header_nonce(header, nonce));
            let hash_u256: U256 = pow_hash.into();
            if hash_u256 > difficulty {
                nonce += 1;
                continue;
            }
            break;
        }
        nonce
    }

    fn generate_nonce() -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen::<u64>();
        rng.gen_range(0, u64::max_value())
    }

    fn process_job(params: String) -> anyhow::Result<u64> {
        let resp: MethodCall = serde_json::from_str(&params)?;
        let params: Params = resp.params.parse()?;
        if let Params::Array(mut values) = params {
            let difficulty: U256 = values
                .pop()
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
                .parse()?;
            let header = values.pop().unwrap().as_str().unwrap().as_bytes().to_vec();
            let nonce = MinerClient::solve(difficulty, &header);
            return Ok(nonce);
        };
        Err(anyhow::Error::msg("mining.notify with bad params"))
    }

    fn submit_job_request(nonce: u64) -> Vec<u8> {
        let mut request = format!(r#"{{"jsonrpc": "2.0", "method": "mining.submit", "params": ["miner1", "", "{:o}"], "id": 7}}"#, nonce).as_bytes().to_vec();
        request.extend(b"\n");
        request
    }

    pub async fn main_loop(addr: SocketAddr) -> Result<()> {
        let mut auth_request =
            r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 2}"#.as_bytes().to_vec();
        auth_request.extend(b"\n");

        let mut auth_response = Vec::<u8>::new();
        let stream = TcpStream::connect(&addr).await.unwrap();
        let stream_arc = Arc::new(stream);
        let reader_arc_clone = stream_arc.clone();
        let writer_arc_clone = stream_arc.clone();
        let mut writer = &*writer_arc_clone;
        writer.write_all(&auth_request).await.unwrap();

        let (tx, mut rx) = mpsc::unbounded();
        let reader_future = async move {
            let mut buf_reader = BufReader::new(&*reader_arc_clone);
            buf_reader
                .read_until(b'\n', &mut auth_response)
                .await
                .unwrap();
            // todo::process auth response
            info!(
                "Reveive miner auth response: {:?}",
                String::from_utf8(auth_response).expect("bad auth resp")
            );
            let mut lines = buf_reader.lines();
            while let Some(line) = lines.next().await {
                let line = line.unwrap();
                info!("Receive the mint job:{}", line.clone());
                let processed = MinerClient::process_job(line);
                if processed.is_err() {
                    continue;
                }
                let nonce = processed.unwrap();
                tx.unbounded_send(nonce).unwrap();
                info!("Process nonce:{:o}", nonce);
            }
        };
        let reader_handle = task::spawn(reader_future);

        let writer_future = async move {
            let mut stream = &*writer_arc_clone;
            while let Some(msg) = rx.next().await {
                info!("Submit nonce is {}", msg);
                let request = MinerClient::submit_job_request(msg);
                stream.write_all(&request).await.unwrap();
            }
        };
        let writer_handle = task::spawn(writer_future);
        reader_handle.await;
        writer_handle.await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::miner::{MineCtx, Miner};
    use crate::miner_client::{verify, MinerClient};
    use crate::stratum::StratumManager;
    use actix_rt::Runtime;
    use bus::BusActor;
    use config::NodeConfig;
    use consensus::argon_consensus::ArgonConsensusHeader;
    use futures_timer::Delay;
    use sc_stratum::{PushWorkHandler, Stratum};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio;
    use types::block::{Block, BlockHeader, BlockTemplate};
    async fn prepare() -> Result<()> {
        let conf = Arc::new(NodeConfig::random_for_test());
        let mut miner = Miner::<ArgonConsensusHeader>::new(BusActor::launch(), conf);
        let stratum = {
            let addr = "127.0.0.1:9000".parse().unwrap();
            let dispatcher = Arc::new(StratumManager::new(miner.clone()));
            Stratum::start(&addr, dispatcher, None).unwrap()
        };
        let mine_ctx = {
            let block = Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test());
            let mut block_template = BlockTemplate::from_block(block);
            block_template.difficult = U256::max_value();
            MineCtx::new(block_template)
        };
        Delay::new(Duration::from_millis(3000)).await;
        miner.set_mint_job(mine_ctx);
        loop {
            stratum.push_work_all(miner.get_mint_job()).unwrap();
            Delay::new(Duration::from_millis(500)).await;
        }
        Ok(())
    }

    #[test]
    fn test_mine() {
        ::logger::init_for_test();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let local = tokio::task::LocalSet::new();
        let fut = actix::System::run_in_tokio("test", &local);
        local.block_on(&mut runtime, async {
            tokio::task::spawn_local(fut);
            tokio::task::spawn_local(prepare());
            Delay::new(Duration::from_millis(500)).await;
            let _ = async_std::future::timeout(
                Duration::from_secs(7),
                MinerClient::main_loop("127.0.0.1:9000".parse().unwrap()),
            )
            .await;
        });
    }

    #[test]
    fn test_hash() {
        let header = "hellostarcoin".as_bytes();
        let df = types::U256::max_value() / 2.into();
        let nonce = MinerClient::solve(df, header.clone());
        let verified = verify(header, nonce, df);
        assert_eq!(true, verified);
    }
}
