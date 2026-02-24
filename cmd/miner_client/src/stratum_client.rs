use crate::stratum_client_service::{
    LoginServiceRequest, ShareRequest, StratumClientService, SubmitSealRequest,
};
use crate::{ConsensusStrategy, JobClient, SealEvent};
use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use futures::future;
use futures::stream::{BoxStream, StreamExt};
use starcoin_logger::prelude::error;
use starcoin_service_registry::ServiceRef;
use starcoin_stratumd::stratum_rpc::LoginRequest;
use starcoin_stratumd::target_hex_to_difficulty;
use starcoin_time_service::TimeService;
use starcoin_types::system_events::{MintBlockEvent, MintEventExtra};
use std::sync::Arc;

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
    async fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let srv = self.stratum_cli_srv.clone();
        let login = self.login.clone();
        let fut = async move {
            let stream = srv
                .send(LoginServiceRequest(login))
                .await?
                .await
                .map_err(|e| anyhow::anyhow!(format!("{}", e)))
                .map(|s| {
                    s.filter_map(|job| {
                        let blob = hex::decode(&job.blob);
                        let diff = target_hex_to_difficulty(&job.target);
                        let extra = job.get_extra();
                        let event = match (blob, diff, extra) {
                            (Ok(blob), Ok(diff), Ok(extra)) => Some(MintBlockEvent {
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
                            }),
                            _ => None,
                        };
                        future::ready(event)
                    })
                    .boxed()
                })?;
            Ok::<BoxStream<MintBlockEvent>, anyhow::Error>(stream.boxed())
        };
        fut.await
    }

    #[allow(clippy::unit_arg)]
    async fn submit_seal(&self, seal: SealEvent) -> Result<()> {
        let srv = self.stratum_cli_srv.clone();
        let mut n = Vec::new();
        n.write_u32::<LittleEndian>(seal.nonce)?;
        let nonce = hex::encode(n);
        let mint_extra = seal
            .extra
            .ok_or_else(|| anyhow::anyhow!("submit missing field"))?;
        if let Err(e) = srv.try_send(SubmitSealRequest(ShareRequest {
            id: mint_extra.worker_id,
            job_id: mint_extra.job_id,
            nonce,
            result: seal.hash_result,
        })) {
            error!("failed to submit seal request {:?}", e);
        }
        Ok(())
    }

    fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
}
