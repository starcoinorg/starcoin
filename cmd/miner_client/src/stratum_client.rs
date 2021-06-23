use starcoin_service_registry::ServiceRef;
use crate::stratum_client_service::{StratumClientService, Request, SubmitSealRequest, ShareRequest};
use crate::{JobClient, SealEvent};
use starcoin_types::block::BlockHeaderExtra;
use async_std::sync::Arc;
use starcoin_types::time::TimeService;
use starcoin_types::system_events::{MintBlockEvent, MintEventExtra};
use anyhow::Result;
use futures::stream::BoxStream;
use starcoin_stratum::rpc::LoginRequest;
use futures::executor::block_on;
use futures::stream::StreamExt;
use starcoin_types::genesis_config::ConsensusStrategy;
use starcoin_stratum::target_hex_to_difficulty;
use futures::future;
use byteorder::{WriteBytesExt, LittleEndian};

#[derive(Clone)]
pub struct StratumJobClient {
    stratum_cli_srv: ServiceRef<StratumClientService>,
    time_service: Arc<dyn TimeService>,

}

impl StratumJobClient {
    pub fn new(stratum_cli_srv: ServiceRef<StratumClientService>, time_service: Arc<dyn TimeService>) -> Self {
        Self {
            stratum_cli_srv,
            time_service,
        }
    }
}

impl JobClient for StratumJobClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let srv = self.stratum_cli_srv.clone();
        let fut = async move {
            let stream = srv.send(LoginRequest {
                login: "fikgol.S10B11021C4F58S10B11021C4F42".to_string(),
                pass: "test".to_string(),
                agent: "Ibctminer/1.0.0".to_string(),
                algo: None,
            }).await?.await.map_err(|e| anyhow::anyhow!(format!("{}",e))).map(|s|
                s.filter_map(|job| {
                    let blob = hex::decode(&job.blob);
                    let diff = target_hex_to_difficulty(&job.target);
                    let extra = job.get_extra();
                    let event =
                        if blob.is_ok() && diff.is_ok() && extra.is_ok() {
                            Some(MintBlockEvent {
                                parent_hash: Default::default(),
                                strategy: ConsensusStrategy::CryptoNight,
                                minting_blob: blob.expect(""),
                                difficulty: diff.expect(""),
                                block_number: job.height,
                                extra: Some(MintEventExtra {
                                    worker_id: job.id,
                                    job_id: job.job_id,
                                    extra: extra.expect(""),
                                }),
                            })
                        } else { None };
                    future::ready(event)
                })
                    .boxed())?;
            Ok::<_, anyhow::Error>(stream.boxed())
        };
        block_on(fut)
    }

    fn submit_seal(&self, seal: SealEvent) -> Result<()> {
        let srv = self.stratum_cli_srv.clone();
        let fut = async move {
            let mut n = Vec::new();
            n.write_u32::<LittleEndian>(seal.nonce)?;
            let nonce = hex::encode(n);
            let mint_extra = seal.extra.ok_or(anyhow::anyhow!("submit missing field"))?;
            let r = srv.send(SubmitSealRequest {
                0: ShareRequest {
                    id: mint_extra.worker_id,
                    job_id: mint_extra.job_id,
                    nonce,
                    result: "84a7d0199aac4fcccf50692aec074878bb124bda15d817174571a52a6a030300".into(),
                },
            }).await?;
            Ok::<_, anyhow::Error>(r)
        };

        block_on(fut)
    }

    fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
}