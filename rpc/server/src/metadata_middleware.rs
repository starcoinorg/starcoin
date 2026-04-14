// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::server::HttpRequest;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::metadata::Metadata;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone, Debug)]
pub struct HttpMetadataLayer {
    ip_headers: Arc<Vec<String>>,
    trust_forwarded_ip_headers: bool,
}

impl HttpMetadataLayer {
    pub fn new(ip_headers: Vec<String>, trust_forwarded_ip_headers: bool) -> Self {
        Self {
            ip_headers: Arc::new(ip_headers),
            trust_forwarded_ip_headers,
        }
    }
}

impl<S> Layer<S> for HttpMetadataLayer {
    type Service = HttpMetadataService<S>;

    fn layer(&self, service: S) -> Self::Service {
        HttpMetadataService {
            service,
            ip_headers: self.ip_headers.clone(),
            trust_forwarded_ip_headers: self.trust_forwarded_ip_headers,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HttpMetadataService<S> {
    service: S,
    ip_headers: Arc<Vec<String>>,
    trust_forwarded_ip_headers: bool,
}

impl<S, B> Service<HttpRequest<B>> for HttpMetadataService<S>
where
    S: Service<HttpRequest<B>> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut request: HttpRequest<B>) -> Self::Future {
        let user = self
            .trust_forwarded_ip_headers
            .then(|| extract_user_from_request(&request, &self.ip_headers))
            .flatten();
        debug!("HTTP RPC metadata: user={:?}, trust_forwarded={}", user, self.trust_forwarded_ip_headers);
        request.extensions_mut().insert(Metadata { user });

        let fut = self.service.call(request);
        Box::pin(fut)
    }
}

fn extract_user_from_request<B>(request: &HttpRequest<B>, ip_headers: &[String]) -> Option<String> {
    for header in ip_headers {
        let Some(value) = request.headers().get(header) else {
            continue;
        };
        let Ok(text) = value.to_str() else {
            continue;
        };
        let Some(raw_ip) = text.split(',').next().map(str::trim) else {
            continue;
        };
        let Ok(ip) = raw_ip.parse::<IpAddr>() else {
            continue;
        };
        return Some(ip.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpsee::server::HttpResponse;
    use std::convert::Infallible;

    #[derive(Clone, Default)]
    struct CaptureMetadataService;

    impl<B: Send + 'static> Service<HttpRequest<B>> for CaptureMetadataService {
        type Response = HttpResponse<()>;
        type Error = Infallible;
        type Future =
            Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: HttpRequest<B>) -> Self::Future {
            let user = request
                .extensions()
                .get::<Metadata>()
                .and_then(|m| m.user.clone());
            let mut response = HttpResponse::new(());
            response.extensions_mut().insert(user);
            Box::pin(async move { Ok(response) })
        }
    }

    #[test]
    fn extract_user_from_first_matching_header() {
        let mut service =
            HttpMetadataLayer::new(vec!["X-Real-IP".into(), "X-Forwarded-For".into()], true)
                .layer(CaptureMetadataService);
        let request = HttpRequest::builder()
            .header("X-Forwarded-For", "10.1.2.3, 8.8.8.8")
            .body(())
            .expect("build request");

        let response = futures::executor::block_on(service.call(request)).expect("service call");
        let user = response
            .extensions()
            .get::<Option<String>>()
            .cloned()
            .expect("response must contain captured user");
        assert_eq!(user, Some("10.1.2.3".to_string()));
    }

    #[test]
    fn ignore_invalid_header_value() {
        let mut service =
            HttpMetadataLayer::new(vec!["X-Real-IP".into()], true).layer(CaptureMetadataService);
        let request = HttpRequest::builder()
            .header("X-Real-IP", "not-an-ip")
            .body(())
            .expect("build request");

        let response = futures::executor::block_on(service.call(request)).expect("service call");
        let user = response
            .extensions()
            .get::<Option<String>>()
            .cloned()
            .expect("response must contain captured user");
        assert_eq!(user, None);
    }

    #[test]
    fn ignore_forwarded_headers_when_not_trusted() {
        let mut service = HttpMetadataLayer::new(vec!["X-Forwarded-For".into()], false)
            .layer(CaptureMetadataService);
        let request = HttpRequest::builder()
            .header("X-Forwarded-For", "10.1.2.3")
            .body(())
            .expect("build request");

        let response = futures::executor::block_on(service.call(request)).expect("service call");
        let user = response
            .extensions()
            .get::<Option<String>>()
            .cloned()
            .expect("response must contain captured user");
        assert_eq!(user, None);
    }
}
