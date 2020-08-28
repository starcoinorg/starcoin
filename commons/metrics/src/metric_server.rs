// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_encoder::JsonEncoder;
use futures::future;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use prometheus::{Encoder, TextEncoder};
use starcoin_logger::prelude::*;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    thread,
};
use tokio::runtime;

fn encode_metrics(encoder: impl Encoder) -> Vec<u8> {
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
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
            // Json encoded libra_metrics;
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

pub fn start_server(host: String, port: u16) {
    // Only called from places that guarantee that host is parsable, but this must be assumed.
    let addr: SocketAddr = (host.as_str(), port)
        .to_socket_addrs()
        .unwrap_or_else(|_| unreachable!("Failed to parse {}:{} as address", host, port))
        .next()
        .unwrap();

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
            .unwrap();
        rt.block_on(async {
            let server = Server::bind(&addr).serve(make_service);
            server.await
        })
        .unwrap();
    });
}
