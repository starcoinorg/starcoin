use crate::stratum_client_service::{ShareRequest, StratumClientService, SubmitSealRequest};
use crate::{JobClient, SealEvent};
use anyhow::Result;
use async_std::sync::Arc;
use byteorder::{LittleEndian, WriteBytesExt};
use futures::executor::block_on;
use futures::future;
use futures::stream::{BoxStream, StreamExt};
use starcoin_service_registry::ServiceRef;
use starcoin_stratum::rpc::LoginRequest;
use starcoin_stratum::target_hex_to_difficulty;
use starcoin_types::genesis_config::ConsensusStrategy;
use starcoin_types::system_events::{MintBlockEvent, MintEventExtra};
use starcoin_types::time::TimeService;

#[derive(Clone)]
pub struct StratumJobClient {
    stratum_cli_srv: ServiceRef<StratumClientService>,
    time_service: Arc<dyn TimeService>,
    login: LoginRequest,
}

impl StratumJobClient {
    pub fn new(
        stratum_cli_srv: ServiceRef<StratumClientService>,
        time_service: Arc<dyn TimeService>,
        login: LoginRequest,
    ) -> Self {
        Self {
            stratum_cli_srv,
            time_service,
            login,
        }
    }
}

impl JobClient for StratumJobClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let srv = self.stratum_cli_srv.clone();
        let login = self.login.clone();
        let fut = async move {
            let stream = srv
                .send(login)
                .await?
                .await
                .map_err(|e| anyhow::anyhow!(format!("{}", e)))
                .map(|s| {
                    s.filter_map(|job| {
                        let blob = hex::decode(&job.blob);
                        let diff = target_hex_to_difficulty(&job.target);
                        let extra = job.get_extra();
                        let event = if let (Ok(blob), Ok(diff), Ok(extra)) = (blob, diff, extra) {
                            Some(MintBlockEvent {
                                parent_hash: Default::default(),
                                strategy: ConsensusStrategy::CryptoNight,
                                minting_blob: blob,
                                difficulty: diff,
                                block_number: job.height,
                                extra: Some(MintEventExtra {
                                    worker_id: job.id,
                                    job_id: job.job_id,

                                    extra,
                                }),
                            })
                        } else {
                            None
                        };
                        future::ready(event)
                    })
                    .boxed()
                })?;
            Ok::<BoxStream<MintBlockEvent>, anyhow::Error>(stream.boxed())
        };
        block_on(fut)
    }

    #[allow(clippy::unit_arg)]
    fn submit_seal(&self, seal: SealEvent) -> Result<()> {
        let srv = self.stratum_cli_srv.clone();
        let fut = async move {
            let mut n = Vec::new();
            n.write_u32::<LittleEndian>(seal.nonce)?;
            let nonce = hex::encode(n);
            let mint_extra = seal
                .extra
                .ok_or_else(|| anyhow::anyhow!("submit missing field"))?;
            let r = srv
                .send(SubmitSealRequest {
                    0: ShareRequest {
                        id: mint_extra.worker_id,
                        job_id: mint_extra.job_id,
                        nonce,
                        result: seal.hash_result,
                    },
                })
                .await?;
            Ok::<(), anyhow::Error>(r)
        };

        block_on(fut)
    }

    fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
}
