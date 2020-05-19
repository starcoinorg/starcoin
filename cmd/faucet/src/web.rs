use crate::{faucet::Faucet, unwrap_or_return};
use anyhow::Result;
use ascii::AsciiString;
use rust_embed::RustEmbed;
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
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

async fn handle_fund(faucet: &Faucet, query: &str) -> Response<Cursor<String>> {
    let query_param =
        unwrap_or_return!(parse_query(query), response_custom(400, "Invalid request"));
    info!("Fund query params: {:?}", query_param);
    let ret = unwrap_or_return!(
        faucet.transfer(
            query_param.amount,
            query_param.address,
            query_param.auth_key
        ),
        response_custom(500, "Inner error")
    );
    if !ret {
        return response_custom(400, "Fund too frequently");
    }
    response_custom(200, "Success")
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
    amount: u64,
    auth_key: Vec<u8>,
}

impl Debug for QueryParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "address: {:?}, amount: {:?}, prefix_key: {:?}",
            self.address,
            self.amount,
            hex::encode(&self.auth_key)
        )
    }
}

fn parse_query(query: &str) -> Result<QueryParam> {
    let mut pairs: Vec<&str> = query.split('&').collect();
    pairs.sort();
    let mut address = "";
    let mut amount = "";
    let mut auth_key = "";
    for pair in pairs {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 {
            match kv[0] {
                "address" => address = kv[1],
                "amount" => amount = kv[1],
                "auth_key" => auth_key = kv[1],
                _ => {}
            };
        }
    }
    let address = AccountAddress::from_str(address)?;
    let amount = u64::from_str(amount)?;
    let auth_key = hex::decode(auth_key).unwrap_or_else(|_| vec![]);
    Ok(QueryParam {
        address,
        amount,
        auth_key,
    })
}
