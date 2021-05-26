use crate::difficulty_to_target_hex;
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
use std::sync::mpsc::TrySendError;
use std::sync::Arc;

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
        let extra = BlockHeaderExtra::default();
        Ok(MinerSubmitSealRequest {
            nonce,
            extra,
            minting_blob: vec![],
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitResult {
    pub result: Status,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeepalivedResult {
    pub result: Status,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub status: String,
}

#[allow(clippy::needless_return)]
#[rpc(server)]
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

#[derive(Debug)]
pub(crate) struct Unsubscribe(pub(crate) SubscriptionId);

impl ServiceRequest for Unsubscribe {
    type Response = ();
}

impl ServiceRequest for SubscribeJobEvent {
    type Response = ();
}

#[derive(Debug, Clone)]
pub(crate) struct SubmitShareEvent(pub(crate) ShareRequest);

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    login: String,
    pass: String,
    agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    algo: Option<Vec<String>>,
}

impl LoginRequest {
    pub fn get_worker_id(&self) -> String {
        let mut hash = DefaultHasher::new(b"");
        hash.update(self.login.as_bytes());
        let output: [u8; 4] = hash.finish().to_vec()[0..4]
            .try_into()
            .expect("Hash len must be 32");
        format!("{}", u32::from_le_bytes(output))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StratumJobResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<LoginRequest>,
    pub id: String,
    pub status: String,
    pub job: StratumJob,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StratumJob {
    pub height: u64,
    pub id: String,
    pub target: String,
    pub job_id: String,
    pub blob: String,
}

impl StratumJobResponse {
    pub fn from(e: &MintBlockEvent, login: Option<LoginRequest>, worker_id: String) -> Self {
        Self {
            login,
            id: worker_id.clone(),
            status: "OK".into(),
            job: StratumJob {
                height: 0,
                id: worker_id,
                target: difficulty_to_target_hex(e.difficulty),
                job_id: hex::encode(&e.minting_blob[0..8]),
                blob: hex::encode(&e.minting_blob),
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
        id: SubscriptionId,
    ) -> jsonrpc_core::Result<bool> {
        match self.service.try_send(Unsubscribe(id)) {
            Ok(()) => Ok(true),
            Err(TrySendError::Full(_)) => Err(jsonrpc_core::Error {
                code: jsonrpc_core::ErrorCode::InternalError,
                message: "stratum service is overloaded".to_string(),
                data: None,
            }),
            Err(TrySendError::Disconnected(_)) => Err(jsonrpc_core::Error {
                code: jsonrpc_core::ErrorCode::InternalError,
                message: "stratum service is down".to_string(),
                data: None,
            }),
        }
    }
}
