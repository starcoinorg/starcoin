use jsonrpsee::{
    batch_response_error,
    core::{server::helpers::prepare_error, JsonRawValue},
    server::{
        middleware::rpc::{
            Batch as RpcBatch, BatchEntry as RpcBatchEntry, BatchEntryErr,
            Notification as RpcNotification, RpcServiceT,
        },
        Extensions,
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
    extensions: Extensions,
}

fn parse_request<'a>(
    data: &'a [u8],
    extensions: &Extensions,
) -> Result<Request<'a>, serde_json::Error> {
    let mut req = serde_json::from_slice::<Request<'a>>(data)?;
    *req.extensions_mut() = extensions.clone();
    Ok(req)
}

fn parse_request_str<'a>(
    data: &'a str,
    extensions: &Extensions,
) -> Result<Request<'a>, serde_json::Error> {
    let mut req = serde_json::from_str::<Request<'a>>(data)?;
    *req.extensions_mut() = extensions.clone();
    Ok(req)
}

fn parse_notification<'a>(
    data: &'a [u8],
    extensions: &Extensions,
) -> Result<Notif<'a>, serde_json::Error> {
    let mut notif = serde_json::from_slice::<Notif<'a>>(data)?;
    *notif.extensions_mut() = extensions.clone();
    Ok(notif)
}

fn parse_notification_str<'a>(
    data: &'a str,
    extensions: &Extensions,
) -> Result<Notif<'a>, serde_json::Error> {
    let mut notif = serde_json::from_str::<Notif<'a>>(data)?;
    *notif.extensions_mut() = extensions.clone();
    Ok(notif)
}

#[instrument(name = "batch", skip(b))]
pub(crate) async fn process_batch_request<S>(b: BatchRequest<S>) -> Option<String>
where
    S: RpcServiceT<MethodResponse = MethodResponse, BatchResponse = BatchResponse> + Send,
{
    let BatchRequest {
        data,
        rpc_service,
        extensions,
    } = b;

    if let Ok(batch) = serde_json::from_slice::<Vec<&JsonRawValue>>(&data) {
        let mut has_call_or_invalid = false;
        let mut has_notification = false;
        let mut entries = Vec::with_capacity(batch.len());

        for value in batch {
            if let Ok(req) = parse_request_str(value.get(), &extensions) {
                has_call_or_invalid = true;
                entries.push(Ok(RpcBatchEntry::Call(req)));
            } else if let Ok(notification) = parse_notification_str(value.get(), &extensions) {
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
    extensions: &Extensions,
) -> Option<MethodResponse>
where
    S: RpcServiceT<MethodResponse = MethodResponse> + Send,
{
    if let Ok(req) = parse_request(&data, extensions) {
        Some(execute_call_with_tracing(req, rpc_service).await)
    } else if parse_notification(&data, extensions).is_ok() {
        None
    } else {
        let (id, code) = prepare_error(&data);
        Some(MethodResponse::error(id, ErrorObject::from(code)).with_extensions(extensions.clone()))
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
    extensions: Extensions,
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

    let res = if matches!(request_kind, Kind::Single) {
        let response = process_single_request(data, &rpc_service, &extensions).await;
        match response {
            Some(response) if response.is_method_call() => Some(response.to_json().to_string()),
            _ => None,
        }
    } else {
        process_batch_request(BatchRequest {
            data,
            rpc_service,
            extensions,
        })
        .await
    };

    drop(conn);

    res
}
