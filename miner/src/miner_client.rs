use actix::{Actor, Context, System};
use actix_rt::Arbiter;
use anyhow::Result;
use async_std::{io::BufReader, net::TcpStream, prelude::*, task};
use futures::channel::mpsc;
use jsonrpc_core::{MethodCall, Params};
use logger::prelude::*;
use serde_json;
use std::marker::PhantomData;
use std::{net::SocketAddr, sync::Arc};
use traits::Consensus;
use types::U256;

pub struct MinerClient<C>
where
    C: Consensus + 'static + Send + Sync,
{
    c_phantom: PhantomData<C>,
}

impl<C> MinerClient<C>
where
    C: Consensus + 'static + Send + Sync,
{
    fn process_job(params: String) -> anyhow::Result<C::ConsensusHeader> {
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
            let nonce = C::solve_consensus_header(&header, difficulty);
            return Ok(nonce);
        };
        Err(anyhow::Error::msg("mining.notify with bad params"))
    }

    fn submit_job_request(consensus_header: C::ConsensusHeader) -> Vec<u8> {
        let nonce: Vec<u8> = consensus_header.into();
        let nonce_hex = hex::encode(&nonce);
        let mut request = format!(r#"{{"jsonrpc": "2.0", "method": "mining.submit", "params": ["miner1", "", {:?}], "id": 7}}"#, nonce_hex).as_bytes().to_vec();
        request.extend(b"\n");
        info!(
            "Submit job response: {:?}",
            &String::from_utf8(request.clone()).unwrap()
        );
        request
    }

    pub async fn run(addr: SocketAddr) -> Result<()> {
        let mut auth_request =
            r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 2}"#.as_bytes().to_vec();
        auth_request.extend(b"\n");

        let mut auth_response = Vec::<u8>::new();
        let stream = TcpStream::connect(&addr).await.unwrap();
        let stream_arc = Arc::new(stream);
        let reader_arc_clone = stream_arc.clone();
        let writer_arc_clone = stream_arc.clone();
        let mut writer = &*writer_arc_clone;
        writer.write_all(&auth_request).await?;

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
                let processed = MinerClient::<C>::process_job(line);
                if processed.is_err() {
                    continue;
                }
                let nonce = processed.unwrap();
                tx.unbounded_send(nonce).unwrap();
            }
        };
        let reader_handle = task::spawn(reader_future);
        let writer_future = async move {
            let mut stream = &*writer_arc_clone;
            while let Some(msg) = rx.next().await {
                let request = MinerClient::<C>::submit_job_request(msg);
                stream.write_all(&request).await.unwrap();
            }
        };
        let writer_handle = task::spawn(writer_future);
        reader_handle.await;
        writer_handle.await;
        Ok(())
    }
}

pub struct MinerClientActor<C>
where
    C: Consensus + 'static + Send + Sync,
{
    addr: SocketAddr,
    c_phantom: PhantomData<C>,
}

impl<C> MinerClientActor<C>
where
    C: Consensus + 'static + Send + Sync,
{
    pub fn new(addr: SocketAddr) -> Self {
        MinerClientActor {
            addr,
            c_phantom: PhantomData,
        }
    }
}

impl<C> Actor for MinerClientActor<C>
where
    C: Consensus + 'static + Send + Sync,
{
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        let addr = self.addr.clone();
        Arbiter::spawn(async move {
            match MinerClient::<C>::run(addr).await {
                Err(e) => {
                    error!("miner client run fail: {}, exist.", e);
                    System::current().stop();
                }
                _ => {}
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::miner::{MineCtx, Miner};
    use crate::miner_client::MinerClient;
    use crate::stratum::StratumManager;
    use actix_rt::System;
    use bus::BusActor;
    use config::NodeConfig;
    use consensus::argon::ArgonConsensus;
    use consensus::argon::ArgonConsensusHeader;
    use futures_timer::Delay;
    use sc_stratum::{PushWorkHandler, Stratum};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio;
    use types::block::{Block, BlockBody, BlockHeader, BlockTemplate};

    async fn prepare() {
        let conf = Arc::new(NodeConfig::random_for_test());
        let mut miner = Miner::<ArgonConsensusHeader>::new(BusActor::launch(), conf);
        let stratum = {
            let addr = "127.0.0.1:9000".parse().unwrap();
            let dispatcher = Arc::new(StratumManager::new(miner.clone()));
            Stratum::start(&addr, dispatcher, None).unwrap()
        };
        let mine_ctx = {
            let header = BlockHeader::default();
            let body = BlockBody::default();
            let block = Block::new(header, body);
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
                MinerClient::<ArgonConsensus>::run("127.0.0.1:9000".parse().unwrap()),
            )
            .await;
        });
    }

    #[test]
    fn test_miner_client_actor() {
        ::logger::init_for_test();
        let mut system = System::new("test");
        system.block_on(async {
            let actor = MinerClientActor::<ArgonConsensus>::new("127.0.0.1:9000".parse().unwrap());
            actor.start();
            let _ = async_std::future::timeout(Duration::from_secs(7), prepare()).await;
        });
    }
}
