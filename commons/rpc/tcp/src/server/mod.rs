use crate::server::connection::{JsonRpcStream, TcpConn, TcpConnDriver};
use crate::server::request::call_with_service;
use crate::server::rpc_service::RpcServiceCfg;

use futures::StreamExt;
use futures_util::future::Either;
use jsonrpsee::{
    core::{middleware::RpcServiceBuilder, JsonRawValue, TEN_MB_SIZE_BYTES},
    server::{
        middleware::rpc::RpcServiceT, stop_channel, BoundedSubscriptions, ConnectionGuard,
        ConnectionPermit, Extensions, IdProvider, MethodResponse, MethodSink, Methods,
        RandomIntegerIdProvider, ServerHandle, StopHandle,
    },
};
use std::{
    fmt,
    future::Future,
    io,
    net::SocketAddr,
    pin::{pin, Pin},
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{net::TcpListener, net::TcpStream, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tower::{layer::util::Identity, Layer, Service};
use tracing::{instrument, trace, Instrument};

mod connection;
mod request;
mod rpc_service;

pub use rpc_service::RpcService as TransportRpcService;

type ConnectionMetadataBuilder = Arc<dyn Fn(SocketAddr) -> Extensions + Send + Sync + 'static>;

pub struct TcpServer<RpcMiddleware = Identity> {
    listener: TcpListener,
    local_addr: SocketAddr,
    id_provider: Arc<dyn IdProvider>,
    cfg: Settings,
    rpc_middleware: RpcServiceBuilder<RpcMiddleware>,
    connection_metadata: ConnectionMetadataBuilder,
}

impl<RpcMiddleware> TcpServer<RpcMiddleware> {
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl<RpcMiddleware> fmt::Debug for TcpServer<RpcMiddleware> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TcpServer")
            .field("local_addr", &self.local_addr)
            .field("cfg", &self.cfg)
            .field("id_provider", &self.id_provider)
            .finish()
    }
}

impl<RpcMiddleware> TcpServer<RpcMiddleware>
where
    RpcMiddleware: Layer<TransportRpcService> + Clone + Send + 'static,
    <RpcMiddleware as Layer<TransportRpcService>>::Service: RpcServiceT<MethodResponse = MethodResponse, BatchResponse = jsonrpsee::BatchResponse>
        + Send
        + Sync
        + 'static,
{
    pub fn start(self, methods: impl Into<Methods>) -> ServerHandle {
        let methods = methods.into();
        let (stop_handle, server_handle) = stop_channel();

        match self.cfg.tokio_runtime.clone() {
            Some(rt) => rt.spawn(self.start_inner(methods, stop_handle)),
            None => tokio::spawn(self.start_inner(methods, stop_handle)),
        };

        server_handle
    }

    async fn start_inner(self, methods: Methods, stop_handle: StopHandle) {
        trace!(local_addr = %self.local_addr, "starting tcp rpc server");

        let mut id: u32 = 0;
        let connection_guard = ConnectionGuard::new(self.cfg.max_connections as usize);
        let stopped = stop_handle.clone().shutdown();
        let mut stopped = pin!(stopped);
        let (drop_on_completion, mut process_connection_awaiter) = mpsc::channel::<()>(1);

        trace!("accepting tcp rpc connections");
        loop {
            match try_accept_conn(&self.listener, stopped).await {
                AcceptConnection::Established {
                    tcp_stream,
                    peer_addr,
                    stop,
                } => {
                    let Some(conn_permit) = connection_guard.try_acquire() else {
                        tracing::warn!(
                            peer = %peer_addr,
                            max_connections = connection_guard.max_connections(),
                            "Dropping TCP RPC connection because the connection limit is reached"
                        );
                        stopped = stop;
                        continue;
                    };

                    let conn_permit = Arc::new(conn_permit);
                    process_connection(ProcessConnection {
                        rpc_middleware: self.rpc_middleware.clone(),
                        conn_permit,
                        conn_id: id,
                        server_cfg: self.cfg.clone(),
                        stop_handle: stop_handle.clone(),
                        drop_on_completion: drop_on_completion.clone(),
                        methods: methods.clone(),
                        id_provider: self.id_provider.clone(),
                        tcp_stream,
                        peer_addr,
                        connection_metadata: self.connection_metadata.clone(),
                    });

                    id = id.wrapping_add(1);
                    stopped = stop;
                }
                AcceptConnection::Shutdown => break,
                AcceptConnection::Err((err, stop)) => {
                    tracing::error!(%err, "Failed accepting a new TCP RPC connection");
                    stopped = stop;
                }
            }
        }

        drop(drop_on_completion);
        while process_connection_awaiter.recv().await.is_some() {}
    }
}

enum AcceptConnection<S> {
    Shutdown,
    Established {
        tcp_stream: TcpStream,
        peer_addr: SocketAddr,
        stop: S,
    },
    Err((io::Error, S)),
}

async fn try_accept_conn<S>(listener: &TcpListener, stopped: S) -> AcceptConnection<S>
where
    S: Future + Unpin,
{
    match futures_util::future::select(pin!(listener.accept()), stopped).await {
        Either::Left((res, stop)) => match res {
            Ok((tcp_stream, peer_addr)) => AcceptConnection::Established {
                tcp_stream,
                peer_addr,
                stop,
            },
            Err(err) => AcceptConnection::Err((err, stop)),
        },
        Either::Right(_) => AcceptConnection::Shutdown,
    }
}

#[derive(Debug, Clone)]
struct ServiceData {
    methods: Methods,
    id_provider: Arc<dyn IdProvider>,
    conn_id: u32,
    conn_permit: Arc<ConnectionPermit>,
    method_sink: MethodSink,
    server_cfg: Settings,
    extensions: Extensions,
}

#[derive(Debug, Clone)]
struct TowerService<L> {
    inner: ServiceData,
    rpc_middleware: RpcServiceBuilder<L>,
}

impl<RpcMiddleware> Service<String> for TowerService<RpcMiddleware>
where
    RpcMiddleware: for<'a> Layer<TransportRpcService>,
    for<'a> <RpcMiddleware as Layer<TransportRpcService>>::Service: Send
        + Sync
        + 'static
        + RpcServiceT<MethodResponse = MethodResponse, BatchResponse = jsonrpsee::BatchResponse>,
{
    type Response = Option<String>;
    type Error = Box<dyn core::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: String) -> Self::Future {
        trace!(request_len = request.len(), "received tcp rpc request");

        let cfg = RpcServiceCfg {
            bounded_subscriptions: BoundedSubscriptions::new(
                self.inner.server_cfg.max_subscriptions_per_connection,
            ),
            id_provider: self.inner.id_provider.clone(),
            sink: self.inner.method_sink.clone(),
        };

        let max_response_body_size = self.inner.server_cfg.max_response_body_size as usize;
        let max_request_body_size = self.inner.server_cfg.max_request_body_size as usize;
        let conn = self.inner.conn_permit.clone();
        let rpc_service = self.rpc_middleware.service(TransportRpcService::new(
            self.inner.methods.clone(),
            max_response_body_size,
            self.inner.conn_id.into(),
            cfg,
        ));
        let extensions = self.inner.extensions.clone();

        let fut = tokio::task::spawn(async move {
            call_with_service(
                request,
                rpc_service,
                max_request_body_size,
                conn,
                extensions,
            )
            .await
        });

        Box::pin(async move { fut.await.map_err(|err| err.into()) })
    }
}

struct ProcessConnection<RpcMiddleware> {
    rpc_middleware: RpcServiceBuilder<RpcMiddleware>,
    conn_permit: Arc<ConnectionPermit>,
    conn_id: u32,
    server_cfg: Settings,
    stop_handle: StopHandle,
    drop_on_completion: mpsc::Sender<()>,
    methods: Methods,
    id_provider: Arc<dyn IdProvider>,
    tcp_stream: TcpStream,
    peer_addr: SocketAddr,
    connection_metadata: ConnectionMetadataBuilder,
}

#[instrument(name = "tcp_rpc_connection", skip_all, fields(conn_id = %params.conn_id, peer = %params.peer_addr))]
fn process_connection<RpcMiddleware>(params: ProcessConnection<RpcMiddleware>)
where
    RpcMiddleware: Layer<TransportRpcService> + Clone + Send + 'static,
    <RpcMiddleware as Layer<TransportRpcService>>::Service: RpcServiceT<MethodResponse = MethodResponse, BatchResponse = jsonrpsee::BatchResponse>
        + Send
        + Sync
        + 'static,
{
    let ProcessConnection {
        rpc_middleware,
        conn_permit,
        conn_id,
        server_cfg,
        stop_handle,
        drop_on_completion,
        id_provider,
        methods,
        tcp_stream,
        peer_addr,
        connection_metadata,
    } = params;

    let tcp = TcpConn(tokio_util::codec::Decoder::framed(
        starcoin_rpc_ipc::stream_codec::StreamCodec::stream_incoming(),
        tcp_stream,
    ));
    let (tx, rx) = mpsc::channel::<Box<JsonRawValue>>(server_cfg.message_buffer_capacity as usize);
    let method_sink = MethodSink::new_with_limit(tx, server_cfg.max_response_body_size);
    let extensions = connection_metadata(peer_addr);
    let service = TowerService {
        inner: ServiceData {
            methods,
            id_provider,
            conn_id,
            conn_permit,
            method_sink,
            server_cfg: server_cfg.clone(),
            extensions,
        },
        rpc_middleware,
    };

    tokio::spawn(async move {
        to_tcp_service(tcp, service, stop_handle, rx)
            .in_current_span()
            .await;
        drop(drop_on_completion);
    });
}

async fn to_tcp_service<S, T>(
    tcp: TcpConn<JsonRpcStream<T>>,
    service: S,
    stop_handle: StopHandle,
    rx: mpsc::Receiver<Box<JsonRawValue>>,
) where
    S: Service<String, Response = Option<String>> + Send + 'static,
    S::Error: Into<Box<dyn core::error::Error + Send + Sync>>,
    S::Future: Send + Unpin,
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let rx_item = ReceiverStream::new(rx);
    let conn = TcpConnDriver {
        conn: tcp,
        service,
        pending_calls: Default::default(),
        items: Default::default(),
    };
    let stopped = stop_handle.shutdown();

    let mut conn = pin!(conn);
    let mut rx_item = pin!(rx_item);
    let mut stopped = pin!(stopped);

    loop {
        tokio::select! {
            _ = &mut conn => break,
            item = rx_item.next() => {
                if let Some(item) = item {
                    conn.push_back(item.to_string());
                }
            }
            _ = &mut stopped => break,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    max_request_body_size: u32,
    max_response_body_size: u32,
    max_connections: u32,
    max_subscriptions_per_connection: u32,
    message_buffer_capacity: u32,
    tokio_runtime: Option<tokio::runtime::Handle>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_request_body_size: TEN_MB_SIZE_BYTES,
            max_response_body_size: TEN_MB_SIZE_BYTES,
            max_connections: 100,
            max_subscriptions_per_connection: 1024,
            message_buffer_capacity: 1024,
            tokio_runtime: None,
        }
    }
}

pub struct Builder<RpcMiddleware> {
    settings: Settings,
    id_provider: Arc<dyn IdProvider>,
    rpc_middleware: RpcServiceBuilder<RpcMiddleware>,
    connection_metadata: ConnectionMetadataBuilder,
}

impl Default for Builder<Identity> {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            id_provider: Arc::new(RandomIntegerIdProvider),
            rpc_middleware: RpcServiceBuilder::new(),
            connection_metadata: Arc::new(|_| Extensions::new()),
        }
    }
}

impl<RpcMiddleware> Builder<RpcMiddleware> {
    pub const fn max_request_body_size(mut self, size: u32) -> Self {
        self.settings.max_request_body_size = size;
        self
    }

    pub const fn max_response_body_size(mut self, size: u32) -> Self {
        self.settings.max_response_body_size = size;
        self
    }

    pub const fn max_connections(mut self, max: u32) -> Self {
        self.settings.max_connections = max;
        self
    }

    pub const fn max_subscriptions_per_connection(mut self, max: u32) -> Self {
        self.settings.max_subscriptions_per_connection = max;
        self
    }

    pub const fn set_message_buffer_capacity(mut self, c: u32) -> Self {
        self.settings.message_buffer_capacity = c;
        self
    }

    pub fn custom_tokio_runtime(mut self, rt: tokio::runtime::Handle) -> Self {
        self.settings.tokio_runtime = Some(rt);
        self
    }

    pub fn set_id_provider<I: IdProvider + 'static>(mut self, id_provider: I) -> Self {
        self.id_provider = Arc::new(id_provider);
        self
    }

    pub fn set_connection_metadata<F>(mut self, metadata_builder: F) -> Self
    where
        F: Fn(SocketAddr) -> Extensions + Send + Sync + 'static,
    {
        self.connection_metadata = Arc::new(metadata_builder);
        self
    }

    pub fn set_rpc_middleware<T>(self, rpc_middleware: RpcServiceBuilder<T>) -> Builder<T> {
        Builder {
            settings: self.settings,
            id_provider: self.id_provider,
            rpc_middleware,
            connection_metadata: self.connection_metadata,
        }
    }

    pub fn build(self, listener: TcpListener) -> io::Result<TcpServer<RpcMiddleware>> {
        let local_addr = listener.local_addr()?;
        Ok(TcpServer {
            listener,
            local_addr,
            id_provider: self.id_provider,
            cfg: self.settings,
            rpc_middleware: self.rpc_middleware,
            connection_metadata: self.connection_metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{SinkExt, StreamExt};
    use jsonrpsee::{server::Extensions, RpcModule};
    use serde_json::{json, Value};
    use tokio::net::TcpStream;
    use tokio_util::codec::Framed;

    #[derive(Clone, Debug)]
    struct TestUser(String);

    async fn start_server(
        module: RpcModule<()>,
        metadata_builder: impl Fn(SocketAddr) -> Extensions + Send + Sync + 'static,
    ) -> (SocketAddr, ServerHandle) {
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .expect("bind tcp listener");
        let addr = listener.local_addr().expect("listener addr");
        let server = Builder::default()
            .set_connection_metadata(metadata_builder)
            .build(listener)
            .expect("build tcp server");
        let handle = server.start(module);
        (addr, handle)
    }

    async fn call(addr: SocketAddr, payload: &str) -> Value {
        let stream = TcpStream::connect(addr).await.expect("connect tcp rpc");
        let mut framed = Framed::new(
            stream,
            starcoin_rpc_ipc::stream_codec::StreamCodec::stream_incoming(),
        );
        framed
            .send(payload.to_string())
            .await
            .expect("send rpc payload");
        let response = framed
            .next()
            .await
            .expect("rpc response frame")
            .expect("rpc response decode");
        serde_json::from_str(&response).expect("valid json-rpc response")
    }

    #[tokio::test]
    async fn serves_raw_jsonrpc_requests_over_tcp() {
        let mut module = RpcModule::new(());
        module
            .register_method("say_hello", |_, _, _| "lo")
            .expect("register method");

        let (addr, handle) = start_server(module, |_| Extensions::new()).await;
        let response = call(addr, r#"{"jsonrpc":"2.0","method":"say_hello","id":1}"#).await;

        assert_eq!(response, json!({"jsonrpc":"2.0","result":"lo","id":1}));
        handle.stop().expect("stop tcp rpc server");
    }

    #[tokio::test]
    async fn injects_connection_metadata_into_requests() {
        let mut module = RpcModule::new(());
        module
            .register_method("whoami", |_, _, extensions| {
                extensions
                    .get::<TestUser>()
                    .map(|user| user.0.clone())
                    .expect("peer metadata")
            })
            .expect("register method");

        let (addr, handle) = start_server(module, |peer_addr| {
            let mut extensions = Extensions::new();
            extensions.insert(TestUser(peer_addr.ip().to_string()));
            extensions
        })
        .await;
        let response = call(addr, r#"{"jsonrpc":"2.0","method":"whoami","id":7}"#).await;

        assert_eq!(
            response,
            json!({"jsonrpc":"2.0","result":"127.0.0.1","id":7})
        );
        handle.stop().expect("stop tcp rpc server");
    }
}
