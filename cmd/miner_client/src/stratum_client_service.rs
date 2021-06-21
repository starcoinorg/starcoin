use actix::clock::Duration;
use anyhow::anyhow;
use anyhow::Result;
use crypto::HashValue;
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures::stream::Fuse;
use futures::{select, Future, Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures_channel::mpsc::UnboundedSender;
use jsonrpc_core;
use jsonrpc_core::{Id, Notification, Params, Version};
use jsonrpc_core_client::RpcError;
use jsonrpc_server_utils::codecs;
use jsonrpc_server_utils::codecs::StreamCodec;
use jsonrpc_server_utils::tokio::macros::support::Poll;
use jsonrpc_server_utils::tokio::net::TcpStream;
use jsonrpc_server_utils::tokio::prelude::io::AsyncWriteExt;
use jsonrpc_server_utils::tokio_util::codec::Decoder;
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starcoin_service_registry::{
    ActorService, EventHandler, RegistryAsyncService, RegistryService, ServiceContext,
    ServiceFactory, ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_stratum::rpc::{
    KeepalivedResult, LoginRequest, ShareRequest, Status, StratumJob, StratumJobResponse,
};
use starcoin_types::block::BlockHeaderExtra;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::Read;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::sync::{atomic, Arc};
use starcoin_config::{MinerClientConfig, NodeConfig};

#[derive(Debug)]
pub enum Request {
    LoginRequest(
        LoginRequest,
        oneshot::Sender<mpsc::UnboundedReceiver<StratumJob>>,
    ),
    SubmitSealRequest(SubmitSealRequest, oneshot::Sender<()>),
}

pub enum PendingRequest {
    LoginRequest(
        oneshot::Sender<mpsc::UnboundedReceiver<StratumJob>>,
    ),
    SubmitSealRequest(oneshot::Sender<()>),
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Response {
    /// A regular JSON-RPC request output (single response).
    Output(Output),
    /// A notification.
    Notification(JobNotification),
    /// Failed
    Failure(Failure),
}

/// Represents jsonrpc request which is a notification.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobNotification {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Option<Version>,
    /// A String containing the name of the method to be invoked.
    pub method: String,
    /// StratumJob
    pub params: StratumJob,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum OutputResponse {
    StratumJob(StratumJobResponse),
    Status(Status),
}

/// Successful response
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    /// Protocol version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<Version>,
    /// Result
    pub result: OutputResponse,
    /// Correlation id
    pub id: u32,
    /// Error
    pub error: Option<ResponseError>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<Version>,
    pub error: ResponseError,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseError {
    pub code: u32,
    pub message: String,
}

impl TryFrom<String> for Response {
    type Error = anyhow::Error;
    fn try_from(resp: String) -> std::result::Result<Self, Self::Error> {
        jsonrpc_core::serde_from_str::<Response>(&resp)
            .map_err(|e| anyhow!(format!("parse response failed: {}", e)))
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MethodCall {
    /// A String specifying the version of the JSON-RPC protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<Version>,
    /// A String containing the name of the method to be invoked.
    pub method: String,
    /// A Structured value that holds the parameter values to be used
    pub params: Params,
    /// An identifier established by the Client
    pub id: u32,
}

impl ServiceRequest for SubmitSealRequest {
    type Response = oneshot::Receiver<()>;
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SubmitSealRequest {
    pub nonce: u32,
    pub extra: BlockHeaderExtra,
    pub minting_blob: Vec<u8>,
}

fn build_request_string<T: ?Sized + Serialize>(
    method: &str,
    argument: &T,
    id: u32,
) -> Result<String> {
    let str = serde_json::to_string(argument)?;
    let params: Params = serde_json::from_str(&str)?;
    let call = MethodCall {
        jsonrpc: None,
        method: method.into(),
        params,
        id,
    };
    let output = serde_json::to_string(&call)?;
    Ok(output)
}

pub struct StratumClientService {
    sender: Option<mpsc::UnboundedSender<Request>>,
    tcp_stream: Option<std::net::TcpStream>,
}

struct Inner {
    request_channel: mpsc::UnboundedReceiver<Request>,
    connections: HashMap<String, mpsc::UnboundedSender<StratumJob>>,
    stream: Option<Pin<Box<Stream<Item=String>>>>,
    pending_requests: HashMap<u32, PendingRequest>,
    sink: Pin<Box<Sink<String, Error=anyhow::Error>>>,
}

impl Inner {
    pub fn new(tcp_stream: TcpStream) -> (Inner, mpsc::UnboundedSender<Request>) {
        let (s, channel) = mpsc::unbounded::<Request>();
        let (sink, stream) = StreamCodec::stream_incoming().framed(tcp_stream).split();
        let sink = Box::pin(sink.sink_map_err(|e| anyhow!(format!("{}", e))));
        let mut stream = Box::pin(
            stream
                .map_err(|e| error!("stratum tcp stream error: {}", e))
                .take_while(|x| futures::future::ready(x.is_ok()))
                .map(|x| x.expect("Stream is closed upon first error")),
        );
        (
            Self {
                connections: Default::default(),
                stream: Some(stream),
                pending_requests: Default::default(),
                sink,
                request_channel: channel,
            },
            s,
        )
    }

    pub async fn process_output(&mut self, response: String) -> Result<()> {
        let resp: Response = response.try_into()?;
        debug!(target: "stratum", "Process resp:{:?}", resp);
        match resp {
            Response::Output(output) => match output.result {
                OutputResponse::StratumJob(job) => {
                    if let Some(pd) = self.pending_requests.remove(&output.id) {
                        match pd {
                            PendingRequest::LoginRequest(pd) => {
                                let (mut s, r) = mpsc::unbounded();
                                s.send(job.job).await?;
                                self.connections.insert(job.id, s);
                                pd.send(r);
                            }
                            PendingRequest::SubmitSealRequest(_) => { return Err(anyhow!("Bad response for stratum login")); }
                        }
                    }
                }
                OutputResponse::Status(status) => {
                    if let Some(pd) = self.pending_requests.remove(&output.id) {
                        match pd {
                            PendingRequest::SubmitSealRequest(sender) => {
                                sender.send(());
                            }
                            PendingRequest::LoginRequest(_) => { return Err(anyhow!("Bad response for stratum submit")); }
                        }
                    }
                }
            },
            Response::Notification(notification) => {
                if let Some(con) = self.connections.get_mut(&notification.params.id) {
                    con.send(notification.params).await?;
                }
            }

            Response::Failure(_) => {}
        }
        Ok(())
    }

    pub async fn start(mut self) {
        let mut stream_fuse = self.stream.take().expect("stream must exist").fuse();
        let mut request_id: u32 = 0;
        loop {
            select! {
                req = self.request_channel.select_next_some() =>{
                    request_id+=1;
                    match req {
                        Request::LoginRequest(login_req, s)=>{
                            let req_str = build_request_string("login", &login_req, request_id).expect("build stratum login request failed never happen");
                            self.sink.send(req_str).await;
                            self.pending_requests.insert(request_id, PendingRequest::LoginRequest(s));
                        }
                        Request::SubmitSealRequest(seal_req,s)=>{
                            let req_str = build_request_string("submit", &seal_req, request_id).expect("build stratum login request failed never happen");
                            self.sink.send(req_str).await;
                            self.pending_requests.insert(request_id, PendingRequest::SubmitSealRequest(s));

                        }
                    }
                },

                resp = stream_fuse.select_next_some() => {
                    self.process_output(resp).await;
                },
            }
        }
    }
}

impl ActorService for StratumClientService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let tcp_stream = TcpStream::from_std(
            self.tcp_stream
                .take()
                .ok_or(anyhow!("stratum client not got a tcp stream"))?,
        )?;
        let (inner, sender) = Inner::new(tcp_stream);
        self.sender = Some(sender);
        ctx.spawn(inner.start());
        Ok(())
    }
}

impl ServiceHandler<StratumClientService, LoginRequest> for StratumClientService {
    fn handle(
        &mut self,
        msg: LoginRequest,
        ctx: &mut ServiceContext<StratumClientService>,
    ) -> <LoginRequest as ServiceRequest>::Response {
        if let Some(sender) = self.sender.clone().take() {
            let (s, r) = futures::channel::oneshot::channel();
            sender.unbounded_send(Request::LoginRequest(msg, s));
            r
        } else {
            unreachable!()
        }
    }
}

impl ServiceHandler<StratumClientService, SubmitSealRequest> for StratumClientService {
    fn handle(
        &mut self,
        msg: SubmitSealRequest,
        ctx: &mut ServiceContext<StratumClientService>,
    ) -> <SubmitSealRequest as ServiceRequest>::Response {
        if let Some(sender) = self.sender.clone().take() {
            let (s, r) = futures::channel::oneshot::channel();
            sender.unbounded_send(Request::SubmitSealRequest(msg, s));
            r
        } else {
            unreachable!()
        }
    }
}

pub struct StratumClientServiceServiceFactory;

impl ServiceFactory<StratumClientService> for StratumClientServiceServiceFactory {
    fn create(ctx: &mut ServiceContext<StratumClientService>) -> Result<StratumClientService> {
        let cfg = ctx.get_shared::<NodeConfig>()?;
        //todo:Fix it
        let addr =cfg.stratum.address.ok_or(anyhow!("stratum server cfg not found"))?;

        let tcp_stream = Some(std::net::TcpStream::connect(&addr.to_string())?);
        Ok(StratumClientService {
            sender: None,
            tcp_stream,
        })
    }
}

#[stest::test(timeout = 120)]
async fn test() {
    let registry = RegistryService::launch();
    let r = registry
        .register_by_factory::<StratumClientService, StratumClientServiceServiceFactory>()
        .await
        .unwrap();
    assert!(registry
        .start_service(StratumClientService::service_name())
        .await
        .is_ok());
    let mut stream = r
        .send(LoginRequest {
            login: "test".to_string(),
            pass: "test".to_string(),
            agent: "test".to_string(),
            algo: None,
        })
        .await
        .unwrap()
        .await
        .unwrap();
    for i in 1..5 {
        select! {
            value=stream.select_next_some()=>{
                println!("receive :{:?}",value);
            }
        }
    }
}
