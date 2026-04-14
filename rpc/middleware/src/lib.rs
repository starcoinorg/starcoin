// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpsee::{
    server::middleware::rpc::{Batch, BatchEntry, Extensions, Notification, Request, RpcServiceT},
    types::Id,
    BatchResponse, MethodResponse,
};
use serde_json::{value::RawValue, Value};
use starcoin_config::ApiSet;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::metadata::Metadata;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::time::Instant;

mod metrics;

pub use metrics::*;

#[derive(Clone, Debug)]
enum CallType {
    MethodCall,
    Notification,
}

impl fmt::Display for CallType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let call_type = match self {
            Self::MethodCall => "method",
            Self::Notification => "notification",
        };
        write!(f, "{}", call_type)
    }
}

struct RpcCallRecord {
    id: String,
    method: String,
    call_type: CallType,
    timer: Instant,
    params: Option<String>,
}

impl RpcCallRecord {
    fn method_call(id: Id<'_>, method: &str, params: Option<&RawValue>) -> Self {
        Self::new(
            id_to_string(&id),
            Some(method.to_owned()),
            CallType::MethodCall,
            params.map(|p| p.get().to_owned()),
        )
    }

    fn notification(method: &str, params: Option<&RawValue>) -> Self {
        Self::new(
            "0".to_owned(),
            Some(method.to_owned()),
            CallType::Notification,
            params.map(|p| p.get().to_owned()),
        )
    }

    fn new(
        id: String,
        method: Option<String>,
        call_type: CallType,
        params: Option<String>,
    ) -> Self {
        let method = method.unwrap_or_default();
        let timer = Instant::now();
        Self {
            id,
            method,
            call_type,
            timer,
            params,
        }
    }

    fn end(self, code: i64, user: Option<String>, metrics: Option<RpcMetrics>) {
        let use_time = self.timer.elapsed();
        let params = if ApiSet::UnsafeContext.check_rpc_method(self.method.as_str()) {
            self.params.unwrap_or_default()
        } else {
            String::new()
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
                .observe(use_time.as_secs_f64());
        }
    }
}

fn id_to_string(id: &Id<'_>) -> String {
    match id {
        Id::Null => String::new(),
        Id::Number(num) => num.to_string(),
        Id::Str(s) => s.to_string(),
    }
}

fn user_from_extensions(ext: &Extensions) -> Option<String> {
    ext.get::<Metadata>().and_then(|m| m.user.clone())
}

fn output_to_code(response: &MethodResponse) -> i64 {
    if response.is_notification() {
        -1
    } else {
        response.as_error_code().map(i64::from).unwrap_or(0)
    }
}

fn parse_id_from_value(id: &Value) -> Option<String> {
    match id {
        Value::Null => Some(String::new()),
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn response_code_by_id(json: &RawValue) -> HashMap<String, i64> {
    let mut code_by_id = HashMap::new();
    let value = match serde_json::from_str::<Value>(json.get()) {
        Ok(v) => v,
        Err(_) => return code_by_id,
    };

    let mut collect_one = |obj: &serde_json::Map<String, Value>| {
        let Some(id) = obj.get("id").and_then(parse_id_from_value) else {
            return;
        };

        let code = obj
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(|c| c.as_i64())
            .unwrap_or(0);
        code_by_id.insert(id, code);
    };

    match value {
        Value::Object(obj) => collect_one(&obj),
        Value::Array(items) => {
            for item in items {
                if let Value::Object(obj) = item {
                    collect_one(&obj);
                }
            }
        }
        _ => {}
    }
    code_by_id
}

trait BatchResponseCodeLookup {
    fn code_by_id(&self) -> Option<HashMap<String, i64>>;
}

impl BatchResponseCodeLookup for MethodResponse {
    fn code_by_id(&self) -> Option<HashMap<String, i64>> {
        Some(response_code_by_id(self.as_json()))
    }
}

impl BatchResponseCodeLookup for BatchResponse {
    fn code_by_id(&self) -> Option<HashMap<String, i64>> {
        let response = MethodResponse::from_batch(self.clone());
        Some(response_code_by_id(response.as_json()))
    }
}

trait NotificationResponseCodeLookup {
    fn code(&self) -> i64;
}

impl NotificationResponseCodeLookup for MethodResponse {
    fn code(&self) -> i64 {
        output_to_code(self)
    }
}

impl NotificationResponseCodeLookup for Option<MethodResponse> {
    fn code(&self) -> i64 {
        self.as_ref().map(output_to_code).unwrap_or(-1)
    }
}

#[derive(Clone)]
pub struct MetricMiddleware<S> {
    service: S,
    metrics: Option<RpcMetrics>,
}

impl<S> MetricMiddleware<S> {
    pub fn new(service: S, metrics: Option<RpcMetrics>) -> Self {
        Self { service, metrics }
    }
}

impl<S> RpcServiceT for MetricMiddleware<S>
where
    S: RpcServiceT<MethodResponse = MethodResponse> + Clone + Send + Sync + 'static,
    S::BatchResponse: BatchResponseCodeLookup,
    S::NotificationResponse: NotificationResponseCodeLookup,
{
    type MethodResponse = MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn call<'a>(
        &self,
        request: Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        let record = RpcCallRecord::method_call(
            request.id(),
            request.method_name(),
            request.params.as_ref().map(|p| p.as_ref()),
        );
        let metrics = self.metrics.clone();
        let user = user_from_extensions(request.extensions());
        let service = self.service.clone();

        async move {
            let response = service.call(request).await;
            record.end(output_to_code(&response), user, metrics);
            response
        }
    }

    fn notification<'a>(
        &self,
        notification: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        let record = RpcCallRecord::notification(
            notification.method_name(),
            notification.params.as_ref().map(|p| p.as_ref()),
        );
        let metrics = self.metrics.clone();
        let user = user_from_extensions(notification.extensions());
        let service = self.service.clone();

        async move {
            let response = service.notification(notification).await;
            record.end(response.code(), user, metrics);
            response
        }
    }

    fn batch<'a>(&self, batch: Batch<'a>) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        let mut method_records = Vec::new();
        let mut notification_records = Vec::new();

        for entry in batch.iter() {
            match entry {
                Ok(BatchEntry::Call(req)) => {
                    method_records.push((
                        id_to_string(&req.id()),
                        RpcCallRecord::method_call(
                            req.id(),
                            req.method_name(),
                            req.params.as_ref().map(|p| p.as_ref()),
                        ),
                        user_from_extensions(req.extensions()),
                    ));
                }
                Ok(BatchEntry::Notification(n)) => {
                    notification_records.push((
                        RpcCallRecord::notification(
                            n.method_name(),
                            n.params.as_ref().map(|p| p.as_ref()),
                        ),
                        user_from_extensions(n.extensions()),
                    ));
                }
                Err(_) => {}
            }
        }

        let service = self.service.clone();
        let metrics = self.metrics.clone();

        async move {
            let response = service.batch(batch).await;
            let code_by_id = response.code_by_id().unwrap_or_default();

            for (id, record, user) in method_records {
                record.end(*code_by_id.get(&id).unwrap_or(&-1), user, metrics.clone());
            }
            for (record, user) in notification_records {
                record.end(-1, user, metrics.clone());
            }

            response
        }
    }
}
