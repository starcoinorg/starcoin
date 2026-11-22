use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use governor::clock::{Clock, DefaultClock};
use governor::state::keyed::DefaultKeyedStateStore;
use governor::state::{InMemoryState, NotKeyed};
use governor::{NotUntil, RateLimiter};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

pub use governor::Quota;

/// Per-user state is kept per key forever unless pruned. Users include per-connection keys, so a
/// client that keeps reconnecting would otherwise grow memory without bound.
const PRUNE_EVERY_CHECKS: u64 = 1 << 16;

type DirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;
type KeyedRateLimiter<K> = RateLimiter<K, DefaultKeyedStateStore<K>, DefaultClock>;

#[derive(Debug)]
pub struct ApiLimiter<User>
where
    User: Clone + Hash + Eq,
{
    global_limiter: DirectRateLimiter,
    user_limiter: KeyedRateLimiter<User>,
}

impl<User> ApiLimiter<User>
where
    User: Clone + Hash + Eq,
{
    pub fn new(global_quota: Quota, user_quota: Quota) -> Self {
        Self {
            global_limiter: DirectRateLimiter::direct(global_quota),
            user_limiter: KeyedRateLimiter::keyed(user_quota),
        }
    }

    pub fn check(
        &self,
        user: Option<&User>,
    ) -> Result<(), NotUntil<<DefaultClock as Clock>::Instant>> {
        if let Some(u) = user {
            self.user_limiter.check_key(u)?;
        }
        self.global_limiter.check()?;
        Ok(())
    }

    /// Drops users whose quota is fully replenished; they behave exactly like new users.
    fn prune_idle_users(&self) {
        self.user_limiter.retain_recent();
        self.user_limiter.shrink_to_fit();
    }
}

#[derive(Debug)]
pub struct ApiLimiters<ApiName, User>
where
    ApiName: Clone + Hash + Eq,
    User: Clone + Hash + Eq,
{
    /// default quota for apis if api quota not specified.
    default_global_api_quota: Quota,
    /// custom api quotas.
    custom_global_api_quotas: HashMap<ApiName, Quota>,
    /// default user quota when calling a api.
    default_user_api_quota: Quota,
    /// custom user quota when calling a api.
    custom_user_api_quotas: HashMap<ApiName, Quota>,

    limiters: DashMap<ApiName, ApiLimiter<User>>,
    checks: AtomicU64,
}

impl<ApiName, User> ApiLimiters<ApiName, User>
where
    ApiName: Clone + Hash + Eq,
    User: Clone + Hash + Eq,
{
    pub fn new(
        default_global_api_quota: Quota,
        custom_global_api_quotas: HashMap<ApiName, Quota>,
        default_user_api_quota: Quota,
        custom_user_api_quotas: HashMap<ApiName, Quota>,
    ) -> Self {
        Self {
            default_global_api_quota,
            custom_global_api_quotas,
            default_user_api_quota,
            custom_user_api_quotas,
            limiters: Default::default(),
            checks: AtomicU64::new(0),
        }
    }

    pub fn check(&self, api: &ApiName, user: Option<&User>) -> Result<(), anyhow::Error> {
        // Prune before taking an entry: iterating the map while holding an entry would deadlock.
        if self.checks.fetch_add(1, Ordering::Relaxed) % PRUNE_EVERY_CHECKS
            == PRUNE_EVERY_CHECKS - 1
        {
            self.prune_idle_users();
        }
        let elem = match self.limiters.entry(api.clone()) {
            Entry::Occupied(o) => o.into_ref(),
            Entry::Vacant(v) => {
                let api_limiter = self.new_limiter(api);
                v.insert(api_limiter)
            }
        };

        elem.check(user).map_err(|e| anyhow::anyhow!("{}", &e))
    }

    fn prune_idle_users(&self) {
        for limiter in self.limiters.iter() {
            limiter.prune_idle_users();
        }
    }

    fn new_limiter(&self, api: &ApiName) -> ApiLimiter<User> {
        let global_quota = self
            .custom_global_api_quotas
            .get(api)
            .cloned()
            .unwrap_or(self.default_global_api_quota);
        let user_quota = self
            .custom_user_api_quotas
            .get(api)
            .cloned()
            .unwrap_or(self.default_user_api_quota);
        ApiLimiter::new(global_quota, user_quota)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ApiLimiter, ApiLimiters, Quota, PRUNE_EVERY_CHECKS};
    use std::collections::HashMap;
    use std::num::NonZeroU32;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_limit() {
        let global_quota = Quota::per_second(unsafe { NonZeroU32::new_unchecked(5) });
        let user_quota = Quota::per_second(unsafe { NonZeroU32::new_unchecked(4) });
        let limiter = ApiLimiter::<String>::new(global_quota, user_quota);
        for _i in 0..4 {
            println!("{}", _i);
            let result = limiter.check(Some(&"abc".to_string()));
            assert!(result.is_ok());
        }
        let result = limiter.check(Some(&"abc".to_string()));
        assert!(result.is_err());
        let result = limiter.check(Some(&"abcd".to_string()));
        assert!(result.is_ok());

        sleep(Duration::from_millis(1000));
        let result = limiter.check(Some(&"abc".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn idle_users_are_pruned() {
        let quota = Quota::per_second(NonZeroU32::new(1000).unwrap());
        let limiters =
            ApiLimiters::<String, u64>::new(quota, HashMap::new(), quota, HashMap::new());
        let api = "chain.info".to_string();
        for user in 0..1000 {
            limiters.check(&api, Some(&user)).unwrap();
        }
        let users = |l: &ApiLimiters<String, u64>| l.limiters.get(&api).unwrap().user_limiter.len();
        assert_eq!(users(&limiters), 1000);

        // Quotas refill within 1 s; the next prune must then drop every idle user.
        sleep(Duration::from_millis(1100));
        for _ in 1000..PRUNE_EVERY_CHECKS {
            limiters.check(&api, None).ok();
        }
        assert_eq!(users(&limiters), 0, "idle per-user state must be pruned");
    }
}
