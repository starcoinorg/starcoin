use anyhow::anyhow;
use anyhow::Result;
use futures::{select, Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures_channel::mpsc;
use futures_channel::oneshot;
use jsonrpc_core::{Params, Version};
use jsonrpc_server_utils::codecs::StreamCodec;
use jsonrpc_server_utils::tokio::net::TcpStream;
use jsonrpc_server_utils::tokio_util::codec::Decoder;
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_config::MinerClientConfig;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
pub use starcoin_stratum::rpc::{
    KeepalivedResult, LoginRequest, ShareRequest, Status, StratumJob, StratumJobResponse,
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::pin::Pin;

#[derive(Debug)]
pub enum Request {
    LoginRequest(
        LoginRequest,
        oneshot::Sender<mpsc::UnboundedReceiver<StratumJob>>,
    ),
    SubmitSealRequest(SubmitSealRequest),
}

pub enum PendingRequest {
    LoginRequest(oneshot::Sender<mpsc::UnboundedReceiver<StratumJob>>),
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
#[allow(clippy::large_enum_variant)]
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
    type Response = ();
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubmitSealRequest(pub ShareRequest);

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
    stream: Option<Pin<Box<dyn Stream<Item = String>>>>,
    pending_requests: HashMap<u32, PendingRequest>,
    sink: Pin<Box<dyn Sink<String, Error = anyhow::Error>>>,
}

impl Inner {
    pub fn new(tcp_stream: TcpStream) -> (Inner, mpsc::UnboundedSender<Request>) {
        let (s, channel) = mpsc::unbounded::<Request>();
        let (sink, stream) = StreamCodec::stream_incoming().framed(tcp_stream).split();
        let sink = Box::pin(sink.sink_map_err(|e| anyhow!(format!("{}", e))));
        let stream = Box::pin(
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
        debug!(target: "stratum", "Process response:{:?}", response);
        let resp: Response = response
            .try_into()
            .map_err(|e| anyhow!("stratum receive invalid types:{}", e))?;

        match resp {
            Response::Output(output) => {
                if let Some(pending_request) = self.pending_requests.remove(&output.id) {
                    match output.result {
                        OutputResponse::StratumJob(job) => {
                            if let PendingRequest::LoginRequest(sender) = pending_request {
                                let (mut s, r) = mpsc::unbounded();
                                s.send(job.job).await?;
                                self.connections.insert(job.id, s);
                                sender
                                    .send(r)
                                    .map_err(|_| anyhow!("write channel failed"))?;
                            }
                        }
                        OutputResponse::Status(status) => {
                            let st = serde_json::to_string(&status)?;
                            info!("stratum got status response:{}", st);
                        }
                    }
                }
            }
            Response::Notification(notification) => {
                if let Some(con) = self.connections.get_mut(&notification.params.id) {
                    con.send(notification.params).await?;
                }
            }

            Response::Failure(e) => {
                error!("stratum client process output request error:{:?}", e);
            }
        }
        Ok(())
    }

    pub async fn start(mut self) {
        let mut stream_fuse = self.stream.take().expect("stream must exist").fuse();
        //move out
        let mut request_id: u32 = 0;
        loop {
            select! {
                req = self.request_channel.select_next_some() =>{
                    request_id+=1;
                    match req {
                        Request::LoginRequest(login_req, s)=>{
                            let req_str = build_request_string("login", &login_req, request_id).expect("build stratum login request failed never happen");
                            debug!("stratum client send request:{}",req_str);
                            if let Err(err) = self.sink.send(req_str).await{
                                error!("stratum send request failed: {}", err);
                                continue
                            }
                            self.pending_requests.insert(request_id, PendingRequest::LoginRequest(s));
                        }
                        Request::SubmitSealRequest(seal_req)=>{
                            let req_str = build_request_string("submit", &seal_req, request_id).expect("build stratum login request failed never happen");
                            debug!("stratum send request:{}",req_str);
                            if let Err(err) = self.sink.send(req_str).await{
                                error!("stratum send request failed: {}", err);
                                continue
                            }
                        }
                    }
                },

                resp = stream_fuse.select_next_some() => {
                    if let Err(err) = self.process_output(resp).await{
                        debug!("process output error:{:?}", err);
                    }

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
                .ok_or_else(|| anyhow!("stratum client not got a tcp stream"))?,
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
        _ctx: &mut ServiceContext<StratumClientService>,
    ) -> <LoginRequest as ServiceRequest>::Response {
        if let Some(sender) = self.sender.clone().take() {
            let (s, r) = futures::channel::oneshot::channel();
            if let Err(err) = sender.unbounded_send(Request::LoginRequest(msg, s)) {
                error!("stratum handle login_request failed: {}", err);
            }
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
        _ctx: &mut ServiceContext<StratumClientService>,
    ) -> <SubmitSealRequest as ServiceRequest>::Response {
        if let Some(sender) = self.sender.clone().take() {
            if let Err(e) = sender.unbounded_send(Request::SubmitSealRequest(msg)) {
                error!("stratum handle submit seal request failed:{}", e);
            }
        } else {
            unreachable!()
        }
    }
}

pub struct StratumClientServiceServiceFactory;

impl ServiceFactory<StratumClientService> for StratumClientServiceServiceFactory {
    fn create(ctx: &mut ServiceContext<StratumClientService>) -> Result<StratumClientService> {
        let cfg = ctx.get_shared::<MinerClientConfig>()?;
        let addr = cfg.server.unwrap_or_else(|| "127.0.0.1:9880".into());
        let tcp_stream = Some(std::net::TcpStream::connect(&addr)?);
        Ok(StratumClientService {
            sender: None,
            tcp_stream,
        })
    }
}
