use futures::StreamExt;
use jsonrpsee::{
    async_client::{Client, ClientBuilder},
    core::client::{ReceivedMessage, TransportReceiverT, TransportSenderT},
};
use starcoin_rpc_ipc::stream_codec::StreamCodec;
use std::{io, time::Duration};
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};
use tokio_util::codec::FramedRead;

#[derive(Debug)]
pub(crate) struct Sender {
    inner: WriteHalf<TcpStream>,
}

impl TransportSenderT for Sender {
    type Error = TcpError;

    async fn send(&mut self, msg: String) -> Result<(), Self::Error> {
        Ok(self.inner.write_all(msg.as_bytes()).await?)
    }

    async fn send_ping(&mut self) -> Result<(), Self::Error> {
        tracing::trace!("send ping - not implemented for raw tcp transport");
        Err(TcpError::NotSupported)
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Receiver {
    inner: FramedRead<tokio::io::ReadHalf<TcpStream>, StreamCodec>,
}

impl TransportReceiverT for Receiver {
    type Error = TcpError;

    async fn receive(&mut self) -> Result<ReceivedMessage, Self::Error> {
        self.inner
            .next()
            .await
            .map_or(Err(TcpError::Closed), |val| Ok(ReceivedMessage::Text(val?)))
    }
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub(crate) struct TcpTransportClientBuilder;

impl TcpTransportClientBuilder {
    pub(crate) async fn build(self, addr: &str) -> Result<(Sender, Receiver), TcpError> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|err| TcpError::FailedToConnect {
                addr: addr.to_string(),
                err,
            })?;
        let (read_half, write_half) = tokio::io::split(stream);
        Ok((
            Sender { inner: write_half },
            Receiver {
                inner: FramedRead::new(read_half, StreamCodec::stream_incoming()),
            },
        ))
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct TcpClientBuilder {
    request_timeout: Duration,
}

impl Default for TcpClientBuilder {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
        }
    }
}

impl TcpClientBuilder {
    pub async fn build(self, addr: &str) -> Result<Client, TcpError> {
        let (tx, rx) = TcpTransportClientBuilder.build(addr).await?;
        Ok(self.build_with_tokio(tx, rx))
    }

    pub fn build_with_tokio<S, R>(self, sender: S, receiver: R) -> Client
    where
        S: TransportSenderT + Send,
        R: TransportReceiverT + Send,
    {
        ClientBuilder::default()
            .request_timeout(self.request_timeout)
            .build_with_tokio(sender, receiver)
    }

    pub const fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TcpError {
    #[error("operation not supported")]
    NotSupported,
    #[error("stream closed")]
    Closed,
    #[error("failed to connect to socket {addr}: {err}")]
    FailedToConnect { addr: String, err: io::Error },
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::Builder;
    use jsonrpsee::{core::client::ClientT, rpc_params, RpcModule};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_connect_and_request() {
        let mut module = RpcModule::new(());
        module
            .register_method("say_hello", |_, _, _| "lo")
            .expect("register method");

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let server = Builder::default().build(listener).expect("build server");
        let handle = server.start(module);

        let client = TcpClientBuilder::default()
            .build(&addr.to_string())
            .await
            .expect("connect tcp client");
        let response: String = client
            .request("say_hello", rpc_params![])
            .await
            .expect("request");

        assert_eq!(response, "lo");
        handle.stop().expect("stop server");
    }
}
