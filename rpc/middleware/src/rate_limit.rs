use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use governor::state::{InMemoryState, NotKeyed};
use governor::{clock::DefaultClock, RateLimiter};
use jsonrpc_core::futures::future::Either;
use jsonrpc_core::futures::Future;
use jsonrpc_core::{Call, Error, ErrorCode, Failure, Id, Metadata, Middleware, Output, Response};
use std::collections::HashMap;
type DirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;
type MethodName = String;

pub use governor::Quota;

#[derive(Debug)]
pub struct JsonApiRateLimitMiddleware {
    default_api_quota: Quota,
    quotas: HashMap<MethodName, Quota>,
    limiter: DashMap<MethodName, DirectRateLimiter>,
}

impl JsonApiRateLimitMiddleware {
    pub fn new(default_api_quota: Quota, custom_api_quotas: HashMap<MethodName, Quota>) -> Self {
        Self {
            default_api_quota,
            quotas: custom_api_quotas,
            limiter: Default::default(),
        }
    }

    fn check_method_call(&self, method_name: &str) -> Result<(), anyhow::Error> {
        let elem = match self.limiter.entry(method_name.to_string()) {
            Entry::Occupied(o) => o.into_ref(),
            Entry::Vacant(v) => {
                let quota = self
                    .quotas
                    .get(method_name)
                    .cloned()
                    .unwrap_or_else(|| self.default_api_quota);
                let api_limiter = DirectRateLimiter::direct(quota);
                v.insert(api_limiter)
            }
        };
        elem.check().map_err(|e| anyhow::anyhow!("{}", &e))
    }
}

impl<M: Metadata> Middleware<M> for JsonApiRateLimitMiddleware {
    type Future = jsonrpc_core::futures::future::FutureResult<Option<Response>, ()>;
    type CallFuture = jsonrpc_core::futures::future::FutureResult<Option<Output>, ()>;

    /// Only override on_call, because we do rate limit on api level, not request level.
    fn on_call<F, X>(&self, call: Call, meta: M, next: F) -> Either<Self::CallFuture, X>
    where
        F: Fn(Call, M) -> X + Send + Sync,
        X: Future<Item = Option<Output>, Error = ()> + Send + 'static,
    {
        let method = match &call {
            Call::MethodCall(m) => Some((m.method.clone(), m.jsonrpc, m.id.clone())),
            Call::Notification(n) => Some((n.method.clone(), n.jsonrpc, Id::Null)),
            Call::Invalid { .. } => None,
        };
        if let Some((m, json_version, id)) = method {
            match self.check_method_call(&m) {
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

#[cfg(test)]
mod tests {
    use crate::{JsonApiRateLimitMiddleware, Quota};
    use failure::_core::num::NonZeroU32;
    use failure::_core::time::Duration;
    use std::collections::HashMap;
    use std::thread::sleep;

    #[test]
    fn test_limit() {
        let quota = Quota::per_second(unsafe { NonZeroU32::new_unchecked(4) });
        let middleware = JsonApiRateLimitMiddleware::new(quota, HashMap::new());
        for _i in 0..4 {
            let result = middleware.check_method_call("abc");
            assert!(result.is_ok());
        }
        let result = middleware.check_method_call("abc");
        assert!(result.is_err());
        sleep(Duration::from_millis(1000));
        let result = middleware.check_method_call("abc");
        assert!(result.is_ok());
    }
}
