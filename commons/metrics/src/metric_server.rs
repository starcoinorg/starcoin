// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_encoder::JsonEncoder;
use futures::future;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use prometheus::{hostname_grouping_key, BasicAuthentication};
use prometheus::{Encoder, TextEncoder};
use starcoin_logger::prelude::*;
use std::{net::SocketAddr, thread};
use tokio::runtime;

fn encode_metrics(encoder: impl Encoder) -> Vec<u8> {
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    //if encode error, just return empty body.
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("Encode metrics error: {:?}", e);
    }
    buffer
}

async fn serve_metrics(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut resp = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            //Prometheus server expects metrics to be on host:port/metrics
            let encoder = TextEncoder::new();
            let buffer = encode_metrics(encoder);
            *resp.body_mut() = Body::from(buffer);
        }
        // expose non-numeric metrics to host:port/json_metrics
        (&Method::GET, "/json_metrics") => {
            // Json encoded diem_metrics;
            let encoder = JsonEncoder;
            let buffer = encode_metrics(encoder);
            *resp.body_mut() = Body::from(buffer);
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Ok(resp)
}

pub fn start_server(addr: SocketAddr) {
    // metric process info.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let process_collector =
            crate::process_collector::ProcessCollector::for_self("starcoin".to_string());
        match process_collector {
            Ok(p) => {
                if let Err(e) = prometheus::register(Box::new(p)) {
                    error!("registry metric collector fail: {:?}", e);
                }
            }
            Err(e) => {
                error!("process_collector error: {:?}", e);
            }
        }
    }

    thread::spawn(move || {
        let make_service =
            make_service_fn(|_| future::ok::<_, hyper::Error>(service_fn(serve_metrics)));

        let mut rt = runtime::Builder::new()
            .basic_scheduler()
            .enable_io()
            .build()
            .expect("build tokio runtime failed");
        if let Err(e) = rt.block_on(async {
            let server = Server::bind(&addr).serve(make_service);
            server.await
        }) {
            error!("Start metric server failed: {:?}", e);
        }
    });
}
pub fn push_metrics(push_server_url: String, auth_username: Option<String>, auth_password: String) {
    let metric_families = prometheus::gather();
    let basic_auth = match auth_username {
        Some(username) => Some(BasicAuthentication {
            username,
            password: auth_password,
        }),
        None => None,
    };
    match prometheus::push_metrics(
        "starcoin_push",
        hostname_grouping_key(),
        &push_server_url,
        metric_families,
        basic_auth,
    ) {
        Ok(_) => {}
        Err(e) => {
            debug!("push metrics error: {:?}", e);
        }
    };
}
