// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::{
    async_client::{Client, ClientBuilder},
    core::client::{ReceivedMessage, TransportReceiverT, TransportSenderT},
    Methods,
};
use std::sync::Arc;
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinError,
};

const SUBSCRIPTION_BUFFER_SIZE: usize = 1024;
const PARSE_ERROR_RESPONSE: &str =
    r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#;

/// Connect a local in-process jsonrpsee client to a local Methods set.
///
/// The returned future should be spawned and awaited to drive the local server loop.
pub fn connect_local(
    methods: impl Into<Methods>,
) -> (
    Client,
    impl std::future::Future<Output = Result<(), LocalRpcError>>,
) {
    let methods = methods.into();

    let (to_server_tx, to_server_rx) = mpsc::unbounded_channel::<String>();
    let (to_client_tx, to_client_rx) = mpsc::unbounded_channel::<String>();

    let client = ClientBuilder::default().build_with_tokio(
        LocalSender { inner: to_server_tx },
        LocalReceiver { inner: to_client_rx },
    );

    let fut = async move {
        run_local_server(methods, to_server_rx, to_client_tx).await;
        Ok(())
    };

    (client, fut)
}

async fn run_local_server(
    methods: Methods,
    mut to_server_rx: UnboundedReceiver<String>,
    to_client_tx: UnboundedSender<String>,
) {
    let to_client_tx = Arc::new(to_client_tx);

    while let Some(req) = to_server_rx.recv().await {
        match methods.raw_json_request(&req, SUBSCRIPTION_BUFFER_SIZE).await {
            Ok((resp, mut sub_rx)) => {
                if to_client_tx.send(resp.get().to_owned()).is_err() {
                    break;
                }

                let sub_tx = Arc::clone(&to_client_tx);
                tokio::spawn(async move {
                    while let Some(msg) = sub_rx.recv().await {
                        if sub_tx.send(msg.get().to_owned()).is_err() {
                            break;
                        }
                    }
                });
            }
            Err(_) => {
                if to_client_tx.send(PARSE_ERROR_RESPONSE.to_string()).is_err() {
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
struct LocalSender {
    inner: UnboundedSender<String>,
}

impl TransportSenderT for LocalSender {
    type Error = LocalRpcError;

    async fn send(&mut self, msg: String) -> Result<(), Self::Error> {
        self.inner.send(msg).map_err(|_| LocalRpcError::ChannelClosed)
    }

    async fn send_ping(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct LocalReceiver {
    inner: UnboundedReceiver<String>,
}

impl TransportReceiverT for LocalReceiver {
    type Error = LocalRpcError;

    async fn receive(&mut self) -> Result<ReceivedMessage, Self::Error> {
        match self.inner.recv().await {
            Some(msg) => Ok(ReceivedMessage::Text(msg)),
            None => Err(LocalRpcError::ChannelClosed),
        }
    }
}

#[derive(Debug, Error)]
pub enum LocalRpcError {
    #[error("local rpc channel closed")]
    ChannelClosed,
    #[error(transparent)]
    Join(#[from] JoinError),
}
