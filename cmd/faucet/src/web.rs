// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::faucet::Faucet;
use anyhow::{bail, Error, Result};
use ascii::AsciiString;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_value::TokenValue;
use starcoin_types::account_config::STCUnit;
use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::str::FromStr;
use tiny_http::{Header, Method, Request, Response, Server};

#[derive(RustEmbed)]
#[folder = "src/static/"]
struct Asset;

fn index_html() -> String {
    let index_html = Asset::get("index.html").unwrap();
    std::str::from_utf8(index_html.as_ref())
        .unwrap()
        .to_string()
}

fn response_custom(status_code: u16, data: String) -> Response<Cursor<String>> {
    let data_len = data.len();
    Response::empty(status_code)
        .with_data(Cursor::new(data), Some(data_len))
        .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
}

fn response_error(error: Error) -> Response<Cursor<String>> {
    warn!("invalid request: {}", error);
    let data = format!("Invalid request: {}", error);
    let data_len = data.len();
    Response::empty(400)
        .with_data(Cursor::new(data), Some(data_len))
        .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap())
}

fn response_ok(resp_json: serde_json::Value) -> Response<Cursor<String>> {
    response_custom(200, resp_json.to_string())
}

fn response(result: Result<serde_json::Value>) -> Response<Cursor<String>> {
    match result {
        Ok(json) => response_ok(json),
        Err(err) => response_error(err),
    }
}

static CONTENT_TYPE: Lazy<Header> = Lazy::new(|| Header {
    field: "Content-Type".parse().unwrap(),
    value: AsciiString::from_ascii("text/html; charset=utf8").unwrap(),
});

pub async fn run(server: Server, faucet: Faucet) {
    for mut request in server.incoming_requests() {
        let pos = request
            .url()
            .find('?')
            .unwrap_or_else(|| request.url().len());
        let url = &request.url()[..pos];
        match url {
            "/" => {
                let response =
                    Response::from_string(index_html()).with_header(CONTENT_TYPE.clone());
                let _err = request.respond(response);
            }
            "/api/fund" => {
                let resp = handle_fund(&faucet, &mut request).await;
                if let Err(err) = request.respond(response(resp).with_header(CONTENT_TYPE.clone()))
                {
                    error!("response err: {}", err)
                }
            }
            _ => {
                let _ = request.respond(response_custom(404, "Not found".to_string()));
            }
        };
    }
}

async fn handle_fund(faucet: &Faucet, request: &mut Request) -> Result<serde_json::Value> {
    info!("fund: {}", request.url());
    debug!("request: {:?}", request);

    if request.method() == &Method::Get {
        bail!("Do not support Get method")
    }
    let mut body = String::new();
    request.as_reader().read_to_string(&mut body)?;
    let fund_request = serde_json::from_str::<FundRequest>(body.as_str())?;
    let amount = fund_request
        .amount
        .and_then(|amount| TokenValue::<STCUnit>::from_str(amount.as_str()).ok());
    let txn_hash = faucet.transfer(amount, fund_request.address)?;
    Ok(serde_json::json!({
       "transaction_id": txn_hash.to_string()
    }))
}

#[derive(Clone, Serialize, Deserialize)]
struct FundRequest {
    address: AccountAddress,
    amount: Option<String>,
}

impl Debug for FundRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "address: {:?}, amount: {:?}", self.address, self.amount,)
    }
}
