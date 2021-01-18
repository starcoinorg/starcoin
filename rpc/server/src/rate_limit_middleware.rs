use jsonrpc_core::futures::future::Either;
use jsonrpc_core::futures::Future;
use jsonrpc_core::{Call, Error, ErrorCode, Failure, Id, Middleware, Output, Response};
type MethodName = String;

use api_limiter::ApiLimiters;
pub use api_limiter::Quota;
use starcoin_config::{ApiQuotaConfig, ApiQuotaConfiguration, QuotaDuration};
use starcoin_rpc_api::metadata::Metadata;

struct QuotaWrapper(Quota);
impl From<ApiQuotaConfig> for QuotaWrapper {
    fn from(c: ApiQuotaConfig) -> Self {
        let q = match c.duration {
            QuotaDuration::Second => Quota::per_second(c.max_burst),
            QuotaDuration::Minute => Quota::per_minute(c.max_burst),
            QuotaDuration::Hour => Quota::per_hour(c.max_burst),
        };
        QuotaWrapper(q)
    }
}

#[derive(Debug)]
pub struct JsonApiRateLimitMiddleware {
    limiters: ApiLimiters<MethodName, String>,
}

impl JsonApiRateLimitMiddleware {
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
        Self { limiters }
    }
}

impl Middleware<Metadata> for JsonApiRateLimitMiddleware {
    type Future = jsonrpc_core::futures::future::FutureResult<Option<Response>, ()>;
    type CallFuture = jsonrpc_core::futures::future::FutureResult<Option<Output>, ()>;

    /// Only override on_call, because we do rate limit on api level, not request level.
    fn on_call<F, X>(&self, call: Call, meta: Metadata, next: F) -> Either<Self::CallFuture, X>
    where
        F: Fn(Call, Metadata) -> X + Send + Sync,
        X: Future<Item = Option<Output>, Error = ()> + Send + 'static,
    {
        let method = match &call {
            Call::MethodCall(m) => Some((m.method.clone(), m.jsonrpc, m.id.clone())),
            Call::Notification(n) => Some((n.method.clone(), n.jsonrpc, Id::Null)),
            Call::Invalid { .. } => None,
        };
        if let Some((m, json_version, id)) = method {
            match self.limiters.check(&m, meta.user.as_ref()) {
                Ok(_) => Either::B(next(call, meta)),
                Err(e) => {
                    let output = Output::Failure(Failure {
                        jsonrpc: json_version,
                        error: Error {
                            code: ErrorCode::ServerError(-10000),
                            message: e.to_string(),
                            data: None,
                        },
                        id,
                    });
                    Either::A(jsonrpc_core::futures::finished(Some(output)))
                }
            }
        } else {
            Either::B(next(call, meta))
        }
    }
}
