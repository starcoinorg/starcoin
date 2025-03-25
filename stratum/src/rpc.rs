use crate::diff_manager::DifficultyManager;
use crate::stratum::Stratum;
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use futures::FutureExt;
use futures::TryFutureExt;
use jsonrpc_core::serde::{Deserialize, Serialize};
use jsonrpc_core::{BoxFuture, ErrorCode, Params, Result};
use jsonrpc_derive::rpc;
use jsonrpc_pubsub::typed::Subscriber;
use jsonrpc_pubsub::{typed, PubSubMetadata, Session, SubscriptionId};
use starcoin_crypto::hash::DefaultHasher;
use starcoin_logger::prelude::*;
use starcoin_miner::SubmitSealRequest as MinerSubmitSealRequest;
use starcoin_service_registry::{ServiceRef, ServiceRequest};
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::MintBlockEvent;
use std::convert::TryInto;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Clone, Default, Debug)]
pub struct Metadata {
    pub session: Option<Arc<Session>>,
    pub user: Option<String>,
}

impl Metadata {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: Some(session),
            user: None,
        }
    }
}

impl jsonrpc_core::Metadata for Metadata {}

impl PubSubMetadata for Metadata {
    fn session(&self) -> Option<Arc<Session>> {
        self.session.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    pub id: String,
    pub job_id: String,
    pub nonce: String,
    pub result: String,
}

impl TryInto<MinerSubmitSealRequest> for ShareRequest {
    type Error = anyhow::Error;
    fn try_into(self) -> anyhow::Result<MinerSubmitSealRequest> {
        let nonce_temp = u32::from_str_radix(self.nonce.as_str(), 16)?;
        let mut n = Vec::new();
        let _ = n.write_u32::<LittleEndian>(nonce_temp);
        let nonce = byteorder::BigEndian::read_u32(&n);
        let extra = hex::decode(self.id)?;
        let extra: [u8; 4] = extra
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to parse extra"))?;
        Ok(MinerSubmitSealRequest {
            nonce,
            extra: BlockHeaderExtra::new(extra),
            minting_blob: vec![],
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitResult {
    pub result: Status,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct KeepalivedResult {
    pub result: Status,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Status {
    pub status: String,
}

#[allow(clippy::needless_return)]
#[rpc]
pub trait StratumRpc {
    type Metadata;
    #[rpc(name = "keepalived", raw_params)]
    fn keepalived(&self, id: Params) -> Result<KeepalivedResult>;

    #[rpc(name = "submit", raw_params)]
    fn submit(&self, share: Params) -> BoxFuture<Result<SubmitResult>>;

    #[pubsub(subscription = "job", subscribe, name = "login", raw_params)]
    fn subscribe(
        &self,
        meta: Self::Metadata,
        subscriber: typed::Subscriber<StratumJobResponse>,
        login: Params,
    );

    #[pubsub(subscription = "job", unsubscribe, name = "logout")]
    fn unsubscribe(
        &self,
        meta: Option<Self::Metadata>,
        id: SubscriptionId,
    ) -> jsonrpc_core::Result<bool>;
}

#[derive(Debug)]
pub(crate) struct SubscribeJobEvent(
    pub(crate) Subscriber<StratumJobResponse>,
    pub(crate) LoginRequest,
);

impl ServiceRequest for SubscribeJobEvent {
    type Response = ();
}

#[derive(Debug, Clone)]
pub struct SubmitShareEvent(pub ShareRequest);

impl ServiceRequest for SubmitShareEvent {
    type Response = anyhow::Result<()>;
}

pub struct StratumRpcImpl {
    service: ServiceRef<Stratum>,
}

impl StratumRpcImpl {
    pub fn new(s: ServiceRef<Stratum>) -> Self {
        Self { service: s }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LoginRequest {
    pub login: String,
    pub pass: String,
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algo: Option<Vec<String>>,
}

impl ServiceRequest for LoginRequest {
    type Response =
        futures::channel::oneshot::Receiver<futures::channel::mpsc::UnboundedReceiver<StratumJob>>;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct WorkerId {
    buff: [u8; 4],
}
impl WorkerId {
    pub fn from_hex(input: String) -> anyhow::Result<Self> {
        let worker_id: [u8; 4] = hex::decode(input)
            .map_err(|_| anyhow::anyhow!("Decode worker id failed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid length of worker id"))?;
        Ok(Self { buff: worker_id })
    }
    pub fn to_hex(&self) -> String {
        hex::encode(self.buff)
    }
}
pub struct MinerWorker {
    pub base_info: LoginRequest,
    pub sub_id: u32,
    pub worker_id: WorkerId,
    pub diff_manager: Arc<RwLock<DifficultyManager>>,
}
impl MinerWorker {
    fn generate_worker_id(login_name: String, sub_id: u32) -> WorkerId {
        let mut hash = DefaultHasher::new(b"");
        hash.update(login_name.as_bytes());
        let mut output: [u8; 4] = hash.finish().to_vec()[0..4]
            .try_into()
            .expect("Hash len should have 8 bytes");
        output
            .iter_mut()
            .zip(u32::to_le_bytes(sub_id).iter())
            .for_each(|(x1, x2)| *x1 ^= *x2);
        WorkerId { buff: output }
    }

    pub fn new(sub_id: u32, base_info: LoginRequest) -> Self {
        let worker_id = Self::generate_worker_id(base_info.login.clone(), sub_id);
        let diff_manager = Arc::new(RwLock::new(DifficultyManager::new()));
        Self {
            base_info,
            sub_id,
            worker_id,
            diff_manager,
        }
    }
    pub fn diff_manager(&self) -> Arc<RwLock<DifficultyManager>> {
        self.diff_manager.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StratumJobResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<LoginRequest>,
    pub id: String,
    pub status: String,
    pub job: StratumJob,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StratumJob {
    pub height: u64,
    pub id: String,
    pub target: String,
    pub job_id: String,
    pub blob: String,
}

impl StratumJob {
    pub fn get_extra(&self) -> anyhow::Result<BlockHeaderExtra> {
        let blob = hex::decode(&self.blob)?;
        if blob.len() != 76 {
            return Err(anyhow::anyhow!("Invalid stratum job"));
        }
        let extra: [u8; 4] = blob[35..39].try_into()?;

        Ok(BlockHeaderExtra::new(extra))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct JobId {
    pub job_id: [u8; 8],
}
impl JobId {
    pub fn from_bob(minting_bob: &[u8]) -> Self {
        let mut job_id = [0u8; 8];
        job_id.copy_from_slice(&minting_bob[0..8]);
        Self { job_id }
    }
    pub fn encode(&self) -> String {
        hex::encode(self.job_id)
    }
    pub fn equal_with(&self, minting_bob: &[u8]) -> bool {
        self.job_id[..] == minting_bob[0..8]
    }
    pub fn new(job_id: &String) -> anyhow::Result<Self> {
        let job_id: [u8; 8] = hex::decode(job_id)
            .map_err(|_| anyhow::anyhow!("Decode job_id failed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid job id with bad length"))?;
        Ok(Self { job_id })
    }
}

impl StratumJobResponse {
    pub fn from(
        e: &MintBlockEvent,
        login: Option<LoginRequest>,
        worker_id: WorkerId,
        target: String,
    ) -> Self {
        let mut minting_blob = e.minting_blob.clone();
        minting_blob[35..39].copy_from_slice(&worker_id.buff);

        let job_id = JobId::from_bob(&e.minting_blob).encode();
        Self {
            login,
            id: worker_id.to_hex(),
            status: "OK".into(),
            job: StratumJob {
                height: 0,
                id: worker_id.to_hex(),
                target,
                job_id,
                blob: hex::encode(&minting_blob),
            },
        }
    }
}

impl StratumRpc for StratumRpcImpl {
    type Metadata = Metadata;

    fn keepalived(&self, _id: Params) -> Result<KeepalivedResult> {
        //TODO: update active time for id
        Ok(KeepalivedResult {
            result: Status {
                status: "KEEPALIVED".to_string(),
            },
        })
    }

    fn submit(&self, share_req: Params) -> BoxFuture<Result<SubmitResult>> {
        let service = self.service.clone();
        let fut = async move {
            let share_params = share_req.parse::<ShareRequest>()?;
            service.send(SubmitShareEvent(share_params)).await??;
            Ok(SubmitResult {
                result: Status {
                    status: "OK".to_string(),
                },
            })
        }
        .map_err(|e: anyhow::Error| jsonrpc_core::Error {
            code: ErrorCode::InvalidParams,
            message: e.to_string(),
            data: None,
        });
        Box::pin(fut.boxed())
    }

    fn subscribe(
        &self,
        _meta: Self::Metadata,
        subscriber: Subscriber<StratumJobResponse>,
        login: Params,
    ) {
        match login.parse::<LoginRequest>() {
            Ok(req) => {
                if let Err(e) = self.service.try_send(SubscribeJobEvent(subscriber, req)) {
                    error!(target: "stratum", "subscribe failed:{}", e)
                }
            }
            Err(e) => {
                let _ = subscriber.reject(e);
            }
        }
    }
    fn unsubscribe(
        &self,
        _meta: Option<Self::Metadata>,
        _id: SubscriptionId,
    ) -> jsonrpc_core::Result<bool> {
        // Not need to implement it
        Ok(false)
    }
}
