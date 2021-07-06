// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{faucet::Faucet, unwrap_or_handle_error};
use anyhow::{Error, Result};
use ascii::AsciiString;
use rust_embed::RustEmbed;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_value::TokenValue;
use starcoin_types::account_config::STCUnit;
use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::str::FromStr;
use tiny_http::{Header, Response, Server};

#[derive(RustEmbed)]
#[folder = "src/static/"]
struct Asset;

fn index_html() -> String {
    let index_html = Asset::get("index.html").unwrap();
    std::str::from_utf8(index_html.as_ref())
        .unwrap()
        .to_string()
}

fn response_custom(status_code: u16, data: &str) -> Response<Cursor<String>> {
    let data_len = data.len();
    Response::empty(status_code)
        .with_data(Cursor::new(data.to_string()), Some(data_len))
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

fn response_ok(txn_hash: HashValue) -> Response<Cursor<String>> {
    let resp_json = serde_json::json!({
       "transaction_id": txn_hash.to_string()
    });
    response_custom(200, resp_json.to_string().as_str())
}

async fn handle_fund(faucet: &Faucet, query: &str) -> Response<Cursor<String>> {
    info!("Fund query: {:?}", query);
    let query_param = unwrap_or_handle_error!(parse_query(query), response_error);

    let txn_hash = unwrap_or_handle_error!(
        faucet.transfer(query_param.amount, query_param.address),
        response_error
    );
    response_ok(txn_hash)
}

pub async fn run(server: Server, faucet: Faucet) {
    for request in server.incoming_requests() {
        let pos = request
            .url()
            .find('?')
            .unwrap_or_else(|| request.url().len());
        let url = &request.url()[..pos];
        let query = request.url()[pos..].trim_start_matches('?');
        match url {
            "/" => {
                let response = Response::from_string(index_html()).with_header(Header {
                    field: "Content-Type".parse().unwrap(),
                    value: AsciiString::from_ascii("text/html; charset=utf8").unwrap(),
                });

                request.respond(response).unwrap();
            }
            "/api/fund" => {
                let resp = handle_fund(&faucet, query).await;
                //todo:: handle io error
                request.respond(resp).unwrap();
            }
            _ => {
                let _ = request.respond(response_custom(404, "Not found"));
            }
        };
    }
}

struct QueryParam {
    address: AccountAddress,
    amount: Option<TokenValue<STCUnit>>,
}

impl Debug for QueryParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "address: {:?}, amount: {:?}", self.address, self.amount,)
    }
}

fn parse_query(query: &str) -> Result<QueryParam> {
    let mut pairs: Vec<&str> = query.split('&').collect();
    pairs.sort_unstable();
    let mut address = "";
    let mut amount = "";
    for pair in pairs {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 {
            match kv[0] {
                "address" => address = kv[1],
                "amount" => amount = kv[1],
                _ => {}
            };
        }
    }
    let address = AccountAddress::from_str(address)?;
    let amount = TokenValue::<STCUnit>::from_str(amount).ok();
    let query_param = QueryParam { address, amount };
    Ok(query_param)
}
