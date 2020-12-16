// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use futures01::{future::Either, Future};
use jsonrpc_core::{Call, Id, Metadata, Middleware, Output, Request, Response};
use starcoin_logger::prelude::*;
use starcoin_metrics::HistogramTimer;
use std::fmt;

mod metrics;

pub use metrics::*;

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
    timer: HistogramTimer,
}

impl RpcCallRecord {
    pub fn new(id: String, method: Option<String>, call_type: CallType) -> Self {
        let method = method.unwrap_or_else(|| "".to_owned());
        let timer = RPC_HISTOGRAMS
            .with_label_values(&[method.as_str()])
            .start_timer();
        Self {
            id,
            method,
            call_type,
            timer,
        }
    }

    pub fn end(self, code: i64) {
        let use_time = self.timer.stop_and_record();

        info!(
            "rpc_call\t{}\t{}\t{}\t{}\t{}",
            self.id, self.call_type, self.method, code, use_time
        );

        RPC_COUNTERS
            .with_label_values(&[
                self.call_type.to_string().as_str(),
                self.method.as_str(),
                &code.to_string(),
            ])
            .inc();
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
        match call {
            Call::MethodCall(method_call) => RpcCallRecord::new(
                id_to_string(&method_call.id),
                Some(method_call.method.clone()),
                CallType::MethodCall,
            ),
            Call::Notification(notification) => RpcCallRecord::new(
                "0".to_owned(),
                Some(notification.method.clone()),
                CallType::Notification,
            ),
            Call::Invalid { id } => RpcCallRecord::new(id_to_string(id), None, CallType::Invalid),
        }
    }
}

pub type BoxFuture = Box<dyn Future<Item = Option<Response>, Error = ()> + Send>;
pub type BoxCallFuture = Box<dyn Future<Item = Option<Output>, Error = ()> + Send>;

#[derive(Clone)]
pub struct MetricMiddleware;

impl<M: Metadata> Middleware<M> for MetricMiddleware {
    type Future = BoxFuture;
    type CallFuture = BoxCallFuture;

    fn on_request<F, X>(&self, request: Request, meta: M, next: F) -> Either<Self::Future, X>
    where
        F: Fn(Request, M) -> X + Send + Sync,
        X: Future<Item = Option<Response>, Error = ()> + Send + 'static,
    {
        Either::B(next(request, meta))
    }

    fn on_call<F, X>(&self, call: Call, meta: M, next: F) -> Either<Self::CallFuture, X>
    where
        F: Fn(Call, M) -> X + Send + Sync,
        X: Future<Item = Option<Output>, Error = ()> + Send + 'static,
    {
        let record: RpcCallRecord = (&call).into();
        let fut = next(call, meta).map(move |output| {
            record.end(output_to_code(output.as_ref()));
            output
        });
        // must declare type to convert type then wrap with Either.
        let box_fut: Self::CallFuture = Box::new(fut);
        Either::A(box_fut)
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
mod tests {
    use super::*;
    use failure::_core::time::Duration;
    use jsonrpc_core::{MetaIoHandler, Params, Value};
    use rand::Rng;
    use starcoin_metrics::get_all_metrics;

    #[stest::test]
    fn test_middleware() {
        let mut io_handler = MetaIoHandler::with_middleware(MetricMiddleware);
        io_handler.add_method("status", |_params: Params| {
            let mut rng = rand::thread_rng();
            let sleep_time = rng.gen_range(1, 50);
            std::thread::sleep(Duration::from_millis(sleep_time));
            Ok(Value::Bool(true))
        });
        let request_tmpl_prefix = r#"{"jsonrpc":"2.0","method":"status","params":[],"id":"#;
        let request_tmpl_suffix = "}";
        //let response = r#"{"jsonrpc":"2.0","result":true,"id":0}"#;
        let count = 10;
        let mut futs = vec![];
        for i in 0..count {
            let request = format!("{}{}{}", request_tmpl_prefix, i, request_tmpl_suffix);
            let fut = io_handler.handle_request(request.as_str(), ());
            futs.push(fut);
        }
        for fut in futs {
            assert!(fut.wait().unwrap().is_some());
        }
        info!("metrics: {:?}", get_all_metrics());
    }
}
