// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_encoder::JsonEncoder;
use futures::future;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use prometheus::{hostname_grouping_key, BasicAuthentication, Registry};
use prometheus::{Encoder, TextEncoder};
use starcoin_logger::prelude::*;
use std::net::SocketAddr;

fn encode_metrics(encoder: impl Encoder, registry: &Registry) -> Vec<u8> {
    let metric_families = registry.gather();
    let mut buffer = vec![];
    //if encode error, just return empty body.
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("Encode metrics error: {:?}", e);
    }
    buffer
}

async fn serve_metrics(
    req: Request<Body>,
    registry: Registry,
) -> Result<Response<Body>, hyper::Error> {
    let mut resp = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            //Prometheus server expects metrics to be on host:port/metrics
            let encoder = TextEncoder::new();
            let buffer = encode_metrics(encoder, &registry);
            *resp.body_mut() = Body::from(buffer);
        }
        // expose non-numeric metrics to host:port/json_metrics
        (&Method::GET, "/json_metrics") => {
            // Json encoded diem_metrics;
            let encoder = JsonEncoder;
            let buffer = encode_metrics(encoder, &registry);
            *resp.body_mut() = Body::from(buffer);
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Ok(resp)
}

pub async fn start_server(addr: SocketAddr, registry: Registry) -> anyhow::Result<()> {
    // metric process info.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let process_collector = crate::process_collector::ProcessCollector::for_self()?;
        if let Err(e) = registry.register(Box::new(process_collector)) {
            warn!("Register process_collector metric failed: {:?}", e);
        }
    }
    let make_service = make_service_fn(|_| {
        let registry = registry.clone();
        future::ok::<_, hyper::Error>(service_fn(move |req| serve_metrics(req, registry.clone())))
    });

    let server = Server::bind(&addr).serve(make_service);
    info!("Metric server started at {}", addr);
    server.await.map_err(|e| e.into())
}

pub fn push_metrics(push_server_url: String, auth_username: Option<String>, auth_password: String) {
    let metric_families = prometheus::gather();
    let basic_auth = auth_username.map(|username| BasicAuthentication {
        username,
        password: auth_password,
    });

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
