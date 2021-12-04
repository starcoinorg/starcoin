// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use futures::{future::Either, Future, FutureExt};
use jsonrpc_core::{Call, FutureResponse, Id, Middleware, Output, Params, Request, Response};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::metadata::Metadata;
use std::fmt;
use std::time::Instant;

mod metrics;

use jsonrpc_core::middleware::NoopCallFuture;
pub use metrics::*;
use starcoin_config::ApiSet;

#[derive(Clone, Debug)]
enum CallType {
    MethodCall,
    Notification,
    Invalid,
}

impl fmt::Display for CallType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            CallType::MethodCall => "method",
            CallType::Notification => "notification",
            CallType::Invalid => "invalid",
        };
        write!(f, "{}", type_str)
    }
}

struct RpcCallRecord {
    id: String,
    method: String,
    call_type: CallType,
    timer: Instant,
    params: Params,
}

impl RpcCallRecord {
    pub fn with_call(call: &Call) -> Self {
        match call {
            Call::MethodCall(method_call) => RpcCallRecord::new(
                id_to_string(&method_call.id),
                Some(method_call.method.clone()),
                CallType::MethodCall,
                method_call.params.clone(),
            ),
            Call::Notification(notification) => RpcCallRecord::new(
                "0".to_owned(),
                Some(notification.method.clone()),
                CallType::Notification,
                notification.params.clone(),
            ),
            Call::Invalid { id } => {
                RpcCallRecord::new(id_to_string(id), None, CallType::Invalid, Params::None)
            }
        }
    }

    pub fn new(id: String, method: Option<String>, call_type: CallType, params: Params) -> Self {
        let method = method.unwrap_or_else(|| "".to_owned());
        let timer = Instant::now();
        Self {
            id,
            method,
            call_type,
            timer,
            params,
        }
    }

    pub fn end(self, code: i64, user: Option<String>, metrics: Option<RpcMetrics>) {
        let use_time = self.timer.elapsed();
        let params = if ApiSet::UnsafeContext.check_rpc_method(self.method.as_str()) {
            serde_json::to_string(&self.params).expect("params should be json")
        } else {
            "".into()
        };

        info!(
            "rpc_call\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.id,
            user.unwrap_or_else(|| "unknown".into()),
            self.call_type,
            self.method,
            code,
            use_time.as_millis(),
            params
        );
        if let Some(metrics) = metrics {
            metrics
                .json_rpc_total
                .with_label_values(&[
                    self.call_type.to_string().as_str(),
                    self.method.as_str(),
                    &code.to_string(),
                ])
                .inc();
            metrics
                .json_rpc_time
                .with_label_values(&[self.method.as_str()])
                .observe(use_time.as_secs_f64())
        }
    }
}

fn id_to_string(id: &Id) -> String {
    match id {
        Id::Null => "".to_owned(),
        Id::Num(num) => num.to_string(),
        Id::Str(str) => str.clone(),
    }
}

impl From<&Call> for RpcCallRecord {
    fn from(call: &Call) -> Self {
        Self::with_call(call)
    }
}

#[derive(Clone)]
pub struct MetricMiddleware {
    metrics: Option<RpcMetrics>,
}

impl MetricMiddleware {
    pub fn new(metrics: Option<RpcMetrics>) -> Self {
        Self { metrics }
    }
}

impl Middleware<Metadata> for MetricMiddleware {
    type Future = FutureResponse;
    type CallFuture = NoopCallFuture;

    fn on_request<F, X>(&self, request: Request, meta: Metadata, next: F) -> Either<Self::Future, X>
    where
        F: Fn(Request, Metadata) -> X + Send + Sync,
        X: Future<Output = Option<Response>> + Send + 'static,
    {
        Either::Right(next(request, meta))
    }

    fn on_call<F, X>(&self, call: Call, meta: Metadata, next: F) -> Either<Self::CallFuture, X>
    where
        F: Fn(Call, Metadata) -> X + Send + Sync,
        X: Future<Output = Option<Output>> + Send + 'static,
    {
        let record: RpcCallRecord = (&call).into();
        let metrics = self.metrics.clone();
        let user_addr = meta.user.clone();
        let fut = next(call, meta).map(move |output| {
            record.end(output_to_code(output.as_ref()), user_addr, metrics);
            output
        });
        // must declare type to convert type then wrap with Either.
        let box_fut: Self::CallFuture = Box::pin(fut);
        Either::Left(box_fut)
    }
}

fn output_to_code(output: Option<&Output>) -> i64 {
    output
        .map(|output| match output {
            Output::Failure(f) => f.error.code.code(),
            Output::Success(_) => 0,
        })
        .unwrap_or(-1)
}

#[cfg(test)]
mod tests;
