// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use api_limiter::ApiLimiters;
pub use api_limiter::Quota;
use jsonrpsee::{
    server::middleware::rpc::{
        Batch, BatchEntry, BatchEntryErr, Extensions, Notification, Request, RpcServiceT,
    },
    types::{ErrorObjectOwned, Id},
    MethodResponse,
};
use starcoin_config::{ApiQuotaConfig, ApiQuotaConfiguration, QuotaDuration};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::metadata::Metadata;
use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;
use tower::Layer;

type MethodName = String;

struct QuotaWrapper(Quota);

impl From<ApiQuotaConfig> for QuotaWrapper {
    fn from(c: ApiQuotaConfig) -> Self {
        let q = match c.duration {
            QuotaDuration::Second => Quota::per_second(c.max_burst),
            QuotaDuration::Minute => Quota::per_minute(c.max_burst),
            QuotaDuration::Hour => Quota::per_hour(c.max_burst),
        };
        Self(q)
    }
}

#[derive(Clone, Debug)]
pub struct JsonApiRateLimitLayer {
    limiters: Arc<ApiLimiters<MethodName, String>>,
    ip_whitelist: Arc<HashSet<String>>,
}

impl JsonApiRateLimitLayer {
    pub fn from_config(quotas: ApiQuotaConfiguration) -> Self {
        let limiters = ApiLimiters::new(
            Into::<QuotaWrapper>::into(quotas.default_global_api_quota()).0,
            quotas
                .custom_global_api_quota()
                .into_iter()
                .map(|(k, v)| (k, Into::<QuotaWrapper>::into(v).0))
                .collect(),
            Into::<QuotaWrapper>::into(quotas.default_user_api_quota()).0,
            quotas
                .custom_user_api_quota()
                .into_iter()
                .map(|(k, v)| (k, Into::<QuotaWrapper>::into(v).0))
                .collect(),
        );
        let ip_whitelist: HashSet<String> = quotas.ip_whitelist().into_iter().collect();
        if ip_whitelist.is_empty() {
            info!("RPC rate limit middleware initialized with no IP whitelist");
        } else {
            info!("RPC rate limit middleware initialized with IP whitelist: {:?}", ip_whitelist);
        }
        Self {
            limiters: Arc::new(limiters),
            ip_whitelist: Arc::new(ip_whitelist),
        }
    }
}

impl<S> Layer<S> for JsonApiRateLimitLayer {
    type Service = JsonApiRateLimitMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        JsonApiRateLimitMiddleware {
            service,
            limiters: self.limiters.clone(),
            ip_whitelist: self.ip_whitelist.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct JsonApiRateLimitMiddleware<S> {
    service: S,
    limiters: Arc<ApiLimiters<MethodName, String>>,
    ip_whitelist: Arc<HashSet<String>>,
}

fn user_from_extensions(extensions: &Extensions) -> Option<String> {
    extensions.get::<Metadata>().and_then(|m| m.user.clone())
}

fn rate_limit_error(err: anyhow::Error) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-10000, err.to_string(), None::<()>)
}

trait NotificationRateLimitResponse {
    fn from_rate_limited(notification: Notification<'_>, err: ErrorObjectOwned) -> Self;
}

impl NotificationRateLimitResponse for MethodResponse {
    fn from_rate_limited(notification: Notification<'_>, err: ErrorObjectOwned) -> Self {
        MethodResponse::error(Id::Null, err).with_extensions(notification.extensions)
    }
}

impl NotificationRateLimitResponse for Option<MethodResponse> {
    fn from_rate_limited(notification: Notification<'_>, err: ErrorObjectOwned) -> Self {
        Some(MethodResponse::error(Id::Null, err).with_extensions(notification.extensions))
    }
}

impl<S> RpcServiceT for JsonApiRateLimitMiddleware<S>
where
    S: RpcServiceT<MethodResponse = MethodResponse> + Clone + Send + Sync + 'static,
    S::NotificationResponse: NotificationRateLimitResponse,
{
    type MethodResponse = MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn call<'a>(
        &self,
        request: Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        let method = request.method_name().to_owned();
        let user = user_from_extensions(request.extensions());
        let service = self.service.clone();
        let limiters = self.limiters.clone();
        let ip_whitelist = self.ip_whitelist.clone();

        async move {
            if let Some(ref ip) = user {
                if ip_whitelist.contains(ip) {
                    debug!("IP {} is whitelisted, bypassing rate limit for method={}", ip, method);
                    return service.call(request).await;
                }
            }
            match limiters.check(&method, user.as_ref()) {
                Ok(_) => service.call(request).await,
                Err(e) => {
                    warn!("Rate limited: method={}, user={:?}, reason={}", method, user, e);
                    MethodResponse::error(request.id(), rate_limit_error(e))
                        .with_extensions(request.extensions)
                }
            }
        }
    }

    fn notification<'a>(
        &self,
        notification: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        let method = notification.method_name().to_owned();
        let user = user_from_extensions(notification.extensions());
        let service = self.service.clone();
        let limiters = self.limiters.clone();
        let ip_whitelist = self.ip_whitelist.clone();

        async move {
            if let Some(ref ip) = user {
                if ip_whitelist.contains(ip) {
                    debug!("IP {} is whitelisted, bypassing rate limit for notification={}", ip, method);
                    return service.notification(notification).await;
                }
            }
            match limiters.check(&method, user.as_ref()) {
                Ok(_) => service.notification(notification).await,
                Err(e) => {
                    warn!("Rate limited: notification={}, user={:?}, reason={}", method, user, e);
                    S::NotificationResponse::from_rate_limited(notification, rate_limit_error(e))
                }
            }
        }
    }

    fn batch<'a>(&self, batch: Batch<'a>) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        let mut entries = Vec::with_capacity(batch.len());
        for entry in batch {
            match entry {
                Ok(BatchEntry::Call(req)) => {
                    let method = req.method_name().to_owned();
                    let user = user_from_extensions(req.extensions());
                    let whitelisted = user.as_ref().map_or(false, |ip| self.ip_whitelist.contains(ip));
                    if whitelisted {
                        debug!("IP {:?} is whitelisted, bypassing rate limit for batch call={}", user, method);
                        entries.push(Ok(BatchEntry::Call(req)));
                    } else {
                        match self.limiters.check(&method, user.as_ref()) {
                            Ok(_) => entries.push(Ok(BatchEntry::Call(req))),
                            Err(e) => {
                                warn!("Rate limited: batch call={}, user={:?}, reason={}", method, user, e);
                                entries.push(Err(BatchEntryErr::new(req.id(), rate_limit_error(e))))
                            }
                        }
                    }
                }
                Ok(BatchEntry::Notification(n)) => {
                    let method = n.method_name().to_owned();
                    let user = user_from_extensions(n.extensions());
                    let whitelisted = user.as_ref().map_or(false, |ip| self.ip_whitelist.contains(ip));
                    if whitelisted {
                        debug!("IP {:?} is whitelisted, bypassing rate limit for batch notification={}", user, method);
                        entries.push(Ok(BatchEntry::Notification(n)));
                    } else {
                        match self.limiters.check(&method, user.as_ref()) {
                            Ok(_) => entries.push(Ok(BatchEntry::Notification(n))),
                            Err(e) => {
                                warn!("Rate limited: batch notification={}, user={:?}, reason={}", method, user, e);
                                entries.push(Err(BatchEntryErr::new(Id::Null, rate_limit_error(e))))
                            }
                        }
                    }
                }
                Err(err) => entries.push(Err(err)),
            }
        }

        self.service.batch(Batch::from(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::str::FromStr;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[derive(Clone, Default)]
    struct ObserveService {
        notifications: Arc<AtomicUsize>,
        batch_errors: Arc<AtomicUsize>,
    }

    impl RpcServiceT for ObserveService {
        type MethodResponse = MethodResponse;
        type NotificationResponse = MethodResponse;
        type BatchResponse = MethodResponse;

        fn call<'a>(
            &self,
            request: Request<'a>,
        ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
            std::future::ready(MethodResponse::error(
                request.id(),
                ErrorObjectOwned::owned(1, "ok", None::<()>),
            ))
        }

        fn notification<'a>(
            &self,
            n: Notification<'a>,
        ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
            let notifications = self.notifications.clone();
            async move {
                notifications.fetch_add(1, Ordering::Relaxed);
                MethodResponse::notification().with_extensions(n.extensions)
            }
        }

        fn batch<'a>(
            &self,
            batch: Batch<'a>,
        ) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
            let batch_errors = self.batch_errors.clone();
            async move {
                let error_count = batch.iter().filter(|entry| entry.is_err()).count();
                batch_errors.store(error_count, Ordering::Relaxed);
                MethodResponse::notification()
            }
        }
    }

    fn test_middleware_for_method(method: &str) -> JsonApiRateLimitMiddleware<ObserveService> {
        let service = ObserveService::default();
        let quotas = ApiQuotaConfiguration {
            custom_global_api_quota: Some(vec![(
                method.to_owned(),
                ApiQuotaConfig::from_str("1/s").expect("valid quota"),
            )]),
            ..Default::default()
        };
        JsonApiRateLimitLayer::from_config(quotas).layer(service)
    }

    fn test_middleware_with_whitelist(
        method: &str,
        whitelist: Vec<String>,
    ) -> JsonApiRateLimitMiddleware<ObserveService> {
        let service = ObserveService::default();
        let quotas = ApiQuotaConfiguration {
            custom_global_api_quota: Some(vec![(
                method.to_owned(),
                ApiQuotaConfig::from_str("1/s").expect("valid quota"),
            )]),
            ip_whitelist: Some(whitelist),
            ..Default::default()
        };
        JsonApiRateLimitLayer::from_config(quotas).layer(service)
    }

    fn request_with_user<'a>(method: &'a str, user: &str) -> Request<'a> {
        let mut req = Request::borrowed(method, None, Id::Number(1));
        req.extensions_mut().insert(Metadata {
            user: Some(user.to_string()),
        });
        req
    }

    #[test]
    fn notification_should_be_rate_limited() {
        let middleware = test_middleware_for_method("state.get");
        let first = Notification::new(Cow::Borrowed("state.get"), None);
        let second = Notification::new(Cow::Borrowed("state.get"), None);

        let first_rsp = futures::executor::block_on(middleware.notification(first));
        let second_rsp = futures::executor::block_on(middleware.notification(second));

        assert!(first_rsp.is_notification());
        assert_eq!(second_rsp.as_error_code(), Some(-10000));
        assert_eq!(middleware.service.notifications.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn batch_notification_should_convert_limited_entry_to_error() {
        let middleware = test_middleware_for_method("txpool.submit_hex_transaction");
        let first_batch = Batch::from(vec![Ok(BatchEntry::Notification(Notification::new(
            Cow::Borrowed("txpool.submit_hex_transaction"),
            None,
        )))]);
        let second_batch = Batch::from(vec![Ok(BatchEntry::Notification(Notification::new(
            Cow::Borrowed("txpool.submit_hex_transaction"),
            None,
        )))]);

        let _ = futures::executor::block_on(middleware.batch(first_batch));
        assert_eq!(middleware.service.batch_errors.load(Ordering::Relaxed), 0);

        let _ = futures::executor::block_on(middleware.batch(second_batch));
        assert_eq!(middleware.service.batch_errors.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn whitelisted_ip_bypasses_rate_limit() {
        let middleware =
            test_middleware_with_whitelist("state.get", vec!["10.0.0.1".to_string()]);

        // First call consumes the quota
        let req1 = request_with_user("state.get", "10.0.0.1");
        let rsp1 = futures::executor::block_on(middleware.call(req1));
        assert_ne!(rsp1.as_error_code(), Some(-10000));

        // Second call should still succeed because 10.0.0.1 is whitelisted
        let req2 = request_with_user("state.get", "10.0.0.1");
        let rsp2 = futures::executor::block_on(middleware.call(req2));
        assert_ne!(rsp2.as_error_code(), Some(-10000));
    }

    #[test]
    fn non_whitelisted_ip_still_rate_limited() {
        let middleware =
            test_middleware_with_whitelist("state.get", vec!["10.0.0.1".to_string()]);

        // First call from non-whitelisted IP
        let req1 = request_with_user("state.get", "192.168.1.1");
        let rsp1 = futures::executor::block_on(middleware.call(req1));
        assert_ne!(rsp1.as_error_code(), Some(-10000));

        // Second call should be rate limited
        let req2 = request_with_user("state.get", "192.168.1.1");
        let rsp2 = futures::executor::block_on(middleware.call(req2));
        assert_eq!(rsp2.as_error_code(), Some(-10000));
    }

    #[test]
    fn whitelisted_ip_bypasses_notification_rate_limit() {
        let middleware =
            test_middleware_with_whitelist("state.get", vec!["10.0.0.1".to_string()]);

        let mut n1 = Notification::new(Cow::Borrowed("state.get"), None);
        n1.extensions_mut().insert(Metadata {
            user: Some("10.0.0.1".to_string()),
        });
        let mut n2 = Notification::new(Cow::Borrowed("state.get"), None);
        n2.extensions_mut().insert(Metadata {
            user: Some("10.0.0.1".to_string()),
        });

        let rsp1 = futures::executor::block_on(middleware.notification(n1));
        let rsp2 = futures::executor::block_on(middleware.notification(n2));

        // Both should succeed (whitelisted)
        assert!(rsp1.is_notification());
        assert!(rsp2.is_notification());
        assert_eq!(middleware.service.notifications.load(Ordering::Relaxed), 2);
    }
}
