use anyhow::anyhow;
use anyhow::Result;
use futures::{select, Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures_channel::mpsc;
use futures_channel::oneshot;
use serde::{Deserialize, Serialize};
use starcoin_config::MinerClientConfig;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_stratum::codec::JsonStreamCodec;
pub use starcoin_stratum::rpc::{
    LoginRequest, ShareRequest, Status, StratumJob, StratumJobResponse,
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_util::codec::Framed;

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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobNotification {
    /// A String specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Option<String>,
    /// A String containing the name of the method to be invoked.
    pub method: String,
    /// StratumJob
    pub params: StratumJob,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum OutputResponse {
    StratumJob(StratumJobResponse),
    Status(Status),
}

/// Successful response
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    /// Protocol version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<String>,
    /// Result
    pub result: OutputResponse,
    /// Correlation id
    pub id: u32,
    /// Error
    pub error: Option<ResponseError>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<String>,
    pub error: ResponseError,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseError {
    pub code: u32,
    pub message: String,
}

impl TryFrom<String> for Response {
    type Error = anyhow::Error;
    fn try_from(resp: String) -> std::result::Result<Self, Self::Error> {
        serde_json::from_str::<Response>(&resp)
            .map_err(|e| anyhow!(format!("parse response failed: {}", e)))
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MethodCall {
    /// A String specifying the version of the JSON-RPC protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<String>,
    /// A String containing the name of the method to be invoked.
    pub method: String,
    /// A Structured value that holds the parameter values to be used
    pub params: serde_json::Value,
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
    let params = serde_json::from_str(&str)?;
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
    stream: Option<Pin<Box<dyn Stream<Item = String> + Send>>>,
    pending_requests: HashMap<u32, PendingRequest>,
    sink: Pin<Box<dyn Sink<String, Error = anyhow::Error> + Send>>,
}

impl Inner {
    pub fn new(tcp_stream: TcpStream) -> (Inner, mpsc::UnboundedSender<Request>) {
        let (s, channel) = mpsc::unbounded::<Request>();
        let (sink, stream) = Framed::new(tcp_stream, JsonStreamCodec::stream_incoming()).split();
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
                            let message = build_request_string("login", &login_req, request_id).expect("build stratum login request failed never happen");
                            debug!("stratum client send request:{}",message);
                            if let Err(err) = self.sink.send(message).await{
                                error!("stratum send request failed: {}", err);
                                continue
                            }
                            self.pending_requests.insert(request_id, PendingRequest::LoginRequest(s));
                        }
                        Request::SubmitSealRequest(seal_req)=>{
                            let message = build_request_string("submit", &seal_req, request_id).expect("build stratum login request failed never happen");
                            debug!("stratum send request:{}",message);
                            if let Err(err) = self.sink.send(message).await{
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
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        let tcp_stream = self
            .tcp_stream
            .take()
            .ok_or_else(|| anyhow!("stratum client not got a tcp stream"))?;
        tcp_stream.set_nonblocking(true)?;
        let tcp_stream = TcpStream::from_std(tcp_stream)?;
        let (inner, sender) = Inner::new(tcp_stream);
        self.sender = Some(sender);
        std::thread::spawn(move || {
            let runtime = Runtime::new().expect("create stratum client tokio runtime");
            runtime.block_on(inner.start());
        });
        Ok(())
    }
}

impl ServiceHandler<StratumClientService, LoginRequest> for StratumClientService {
    fn handle(
        &mut self,
        msg: LoginRequest,
        _ctx: &mut ServiceContext<StratumClientService>,
    ) -> <LoginRequest as ServiceRequest>::Response {
        match self.sender.clone() {
            Some(sender) => {
                let (s, r) = futures::channel::oneshot::channel();
                if let Err(err) = sender.unbounded_send(Request::LoginRequest(msg, s)) {
                    error!("stratum handle login_request failed: {}", err);
                }
                r
            }
            _ => {
                unreachable!()
            }
        }
    }
}

impl ServiceHandler<StratumClientService, SubmitSealRequest> for StratumClientService {
    fn handle(
        &mut self,
        msg: SubmitSealRequest,
        _ctx: &mut ServiceContext<StratumClientService>,
    ) -> <SubmitSealRequest as ServiceRequest>::Response {
        //FIXME: Failed to receive this msg since upgrade actix to 0.13.
        match self.sender.clone() {
            Some(sender) => {
                if let Err(e) = sender.unbounded_send(Request::SubmitSealRequest(msg)) {
                    error!("stratum handle submit seal request failed:{}", e);
                }
            }
            _ => {
                unreachable!()
            }
        }
    }
}

pub struct StratumClientServiceServiceFactory;

impl ServiceFactory<StratumClientService> for StratumClientServiceServiceFactory {
    fn create(ctx: &mut ServiceContext<StratumClientService>) -> Result<StratumClientService> {
        let cfg = ctx.get_shared::<MinerClientConfig>()?;
        let addr = cfg.server.unwrap_or_else(|| "127.0.0.1:9880".into());
        let tcp_stream = Some(std::net::TcpStream::connect(addr)?);
        Ok(StratumClientService {
            sender: None,
            tcp_stream,
        })
    }
}
