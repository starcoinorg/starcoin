// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::{anyhow, ensure, format_err, Result};
use async_std::{io::BufReader, net::TcpStream, prelude::*, task};
use config::MinerConfig;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
pub use jsonrpc_core::types::{
    id, request, response, version, Error as Jsonrpc_err, Id, MethodCall, Params, Value, Version,
};
use jsonrpc_core::{Output, Response};
use logger::prelude::*;
use serde_json::error::Error as JsonError;
use serde_json::{self, json};
use std::sync::Arc;
use thiserror::Error;
use types::U256;

pub struct StratumClient {
    request_tx: mpsc::UnboundedSender<Vec<u8>>,
    tcp_stream: Arc<TcpStream>,
}

impl StratumClient {
    pub fn new(config: &MinerConfig) -> Result<Self> {
        let tcp_stream =
            task::block_on(async { TcpStream::connect(&config.stratum_server).await })?;
        let tcp_stream = Arc::new(tcp_stream);
        let (request_tx, mut request_rx) = mpsc::unbounded::<Vec<u8>>();
        let writer = tcp_stream.clone();
        task::spawn(async move {
            let mut stream = &*writer;
            while let Some(request) = request_rx.next().await {
                if let Err(e) = stream.write_all(&request).await {
                    error!("Failed to write stream in stratum client:{:?}", e)
                }
            }
        });
        Ok(Self {
            request_tx,
            tcp_stream,
        })
    }

    pub async fn subscribe(&mut self) -> Result<mpsc::UnboundedReceiver<(Vec<u8>, U256)>> {
        let (mut job_tx, job_rx) = mpsc::unbounded();
        let tcp_stream = self.tcp_stream.clone();
        let authed = self.auth(tcp_stream.clone()).await?;
        if !authed {
            return Err(anyhow::anyhow!("Stratum client auth failed"));
        }
        info!("Stratum client auth succeeded");
        let reader_fut = async move {
            let reader = BufReader::new(&*tcp_stream);
            let mut lines = reader.lines();
            while let Some(line) = lines.next().await {
                match line {
                    Ok(request) => {
                        if let Ok(response_ok) = parse_response::<bool>(&request) {
                            if !response_ok {
                                error!("stratum received server respond false");
                            }
                            continue;
                        };
                        debug!("Receive from stratum: {}", &request);
                        match process_request(request.as_str()) {
                            Ok(job) => {
                                if let Err(e) = job_tx.send(job).await {
                                    error!("stratum subscribe job tx send failed:{:?}", e);
                                }
                            }
                            Err(err) => error!("Process request {:?} error: {:?}", request, err),
                        }
                    }
                    Err(err) => error!(
                        "Stratum client Failed to read request from tcp stream:{:?}",
                        err
                    ),
                }
            }
        };
        task::spawn(reader_fut);
        //TODO: auth failed
        Ok(job_rx)
    }

    pub async fn submit_seal(&mut self, seal: (Vec<u8>, u64)) -> Result<()> {
        let (_pow_header, nonce) = seal;
        let nonce_hex = format!("{:x}", nonce);
        let params = vec![json!(0), json!(0), json!(nonce_hex)];
        let method = "mining.submit".to_owned();
        self.request(method, params, 0).await?;
        Ok(())
    }

    async fn auth(&mut self, tcp_stream: Arc<TcpStream>) -> Result<bool> {
        let params = vec![json!("miner"), json!("")];
        let method = "mining.authorize".to_owned();
        self.request(method, params, 0).await?;
        let mut auth_response = String::new();
        BufReader::new(&*tcp_stream)
            .read_line(&mut auth_response)
            .await?;
        debug!("auth response: {}", auth_response);
        let authed = parse_response::<bool>(auth_response.as_str())?;
        Ok(authed)
    }

    async fn request(&mut self, method: String, params: Vec<Value>, id: u64) -> Result<()> {
        let call = MethodCall {
            method,
            params: Params::Array(params),
            jsonrpc: Some(Version::V2),
            id: Id::Num(id),
        };
        let mut req = serde_json::to_vec(&call)?;
        req.extend(b"\n");
        info!("Request to stratum: {}", String::from_utf8(req.clone())?);
        self.request_tx.send(req).await?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum StratumError {
    #[error("json error")]
    Json(#[from] JsonError),
    #[error("rpc failed")]
    Fail(Jsonrpc_err),
}

pub(crate) fn parse_response<T: serde::de::DeserializeOwned>(
    resp: &str,
) -> Result<T, StratumError> {
    let response = Response::from_json(resp).map_err(StratumError::Json)?;
    match response {
        Response::Single(output) => match output {
            Output::Success(success) => {
                serde_json::from_value::<T>(success.result).map_err(StratumError::Json)
            }
            Output::Failure(failure) => Err(StratumError::Fail(failure.error)),
        },
        Response::Batch(outputs) => {
            error!("Unsupported batch response: {:?}", outputs);
            Err(StratumError::Fail(Jsonrpc_err::parse_error()))
        }
    }
}

pub(crate) fn process_request(req: &str) -> Result<(Vec<u8>, U256)> {
    let value = serde_json::from_str::<Value>(req).map_err(StratumError::Json)?;
    let request = serde_json::from_value::<MethodCall>(value).map_err(StratumError::Json)?;
    let params: Params = request.params.parse()?;
    if let Params::Array(values) = params {
        ensure!(values.len() == 2, "Invalid mint request params");
        let header = values[0]
            .as_str()
            .ok_or_else(|| format_err!("Invalid header field in request"))
            .and_then(|h| {
                hex::decode(h).map_err(|_| anyhow!("Invalid header field with bad hex encode"))
            })?;
        ensure!(header.len() == 32, "Invalid header length");

        let difficulty = values[1]
            .as_str()
            .ok_or_else(|| format_err!("Invalid difficulty field in request"))
            .and_then(|d| {
                d.to_owned()
                    .parse::<U256>()
                    .map_err(|e| anyhow!(e.to_string()))
            })?;
        return Ok((header, difficulty));
    }
    Err(anyhow!("mining.notify with bad params"))
}
