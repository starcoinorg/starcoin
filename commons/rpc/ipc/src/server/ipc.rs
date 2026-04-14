//! IPC request handling adapted from [`jsonrpsee`] http request handling

use jsonrpsee::{
    batch_response_error,
    core::{server::helpers::prepare_error, JsonRawValue},
    server::middleware::rpc::{
        Batch as RpcBatch, BatchEntry as RpcBatchEntry, BatchEntryErr,
        Notification as RpcNotification, RpcServiceT,
    },
    types::{
        error::{reject_too_big_request, ErrorCode},
        ErrorObject, Id, InvalidRequest, Request,
    },
    BatchResponse, MethodResponse,
};
use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;
use tracing::instrument;

type Notif<'a> = RpcNotification<'a>;

#[derive(Debug, Clone)]
pub(crate) struct BatchRequest<S> {
    data: Vec<u8>,
    rpc_service: S,
}

// Batch responses must be sent back as a single message so we read the results from each
// request in the batch and read the results off of a new channel, `rx_batch`, and then send the
// complete batch response back to the client over `tx`.
#[instrument(name = "batch", skip(b))]
pub(crate) async fn process_batch_request<S>(b: BatchRequest<S>) -> Option<String>
where
    S: RpcServiceT<MethodResponse = MethodResponse, BatchResponse = BatchResponse> + Send,
{
    let BatchRequest { data, rpc_service } = b;

    if let Ok(batch) = serde_json::from_slice::<Vec<&JsonRawValue>>(&data) {
        let mut has_call_or_invalid = false;
        let mut has_notification = false;
        let mut entries = Vec::with_capacity(batch.len());

        for value in batch {
            if let Ok(req) = serde_json::from_str::<Request<'_>>(value.get()) {
                has_call_or_invalid = true;
                entries.push(Ok(RpcBatchEntry::Call(req)));
            } else if let Ok(notification) = serde_json::from_str::<Notif<'_>>(value.get()) {
                has_notification = true;
                entries.push(Ok(RpcBatchEntry::Notification(notification)));
            } else {
                has_call_or_invalid = true;
                let id = match serde_json::from_str::<InvalidRequest<'_>>(value.get()) {
                    Ok(err) => err.id,
                    Err(_) => Id::Null,
                };
                entries.push(Err(BatchEntryErr::new(
                    id,
                    ErrorObject::from(ErrorCode::InvalidRequest),
                )));
            }
        }

        let response = rpc_service.batch(RpcBatch::from(entries)).await;
        if has_notification && !has_call_or_invalid {
            None
        } else {
            Some(MethodResponse::from_batch(response).to_json().to_string())
        }
    } else {
        Some(batch_response_error(Id::Null, ErrorObject::from(ErrorCode::ParseError)).to_string())
    }
}

pub(crate) async fn process_single_request<S>(
    data: Vec<u8>,
    rpc_service: &S,
) -> Option<MethodResponse>
where
    S: RpcServiceT<MethodResponse = MethodResponse> + Send,
{
    if let Ok(req) = serde_json::from_slice::<Request<'_>>(&data) {
        Some(execute_call_with_tracing(req, rpc_service).await)
    } else if serde_json::from_slice::<Notif<'_>>(&data).is_ok() {
        None
    } else {
        let (id, code) = prepare_error(&data);
        Some(MethodResponse::error(id, ErrorObject::from(code)))
    }
}

#[instrument(name = "method_call", fields(method = req.method.as_ref()), skip(req, rpc_service))]
pub(crate) async fn execute_call_with_tracing<S>(
    req: Request<'_>,
    rpc_service: &S,
) -> MethodResponse
where
    S: RpcServiceT<MethodResponse = MethodResponse> + Send,
{
    rpc_service.call(req).await
}

pub(crate) async fn call_with_service<S>(
    request: String,
    rpc_service: S,
    max_request_body_size: usize,
    conn: Arc<OwnedSemaphorePermit>,
) -> Option<String>
where
    S: RpcServiceT<MethodResponse = MethodResponse, BatchResponse = BatchResponse> + Send,
{
    enum Kind {
        Single,
        Batch,
    }

    let request_kind = request
        .chars()
        .find_map(|c| match c {
            '{' => Some(Kind::Single),
            '[' => Some(Kind::Batch),
            _ => None,
        })
        .unwrap_or(Kind::Single);

    let data = request.into_bytes();
    if data.len() > max_request_body_size {
        return Some(
            batch_response_error(
                Id::Null,
                reject_too_big_request(max_request_body_size as u32),
            )
            .to_string(),
        );
    }

    // Single request or notification
    let res = if matches!(request_kind, Kind::Single) {
        let response = process_single_request(data, &rpc_service).await;
        match response {
            Some(response) if response.is_method_call() => Some(response.to_json().to_string()),
            _ => {
                // subscription responses are sent directly over the sink, return a response here
                // would lead to duplicate responses for the subscription response
                None
            }
        }
    } else {
        process_batch_request(BatchRequest { data, rpc_service }).await
    };

    drop(conn);

    res
}
