// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::{MetaIoHandler, RemoteProcedure};
use starcoin_config::{Api, ApiQuotaConfig, ApiQuotaConfiguration, QuotaDuration};
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_middleware::{JsonApiRateLimitMiddleware, MetricMiddleware, Quota};
use std::collections::HashMap;

type Middlewares = (MetricMiddleware, JsonApiRateLimitMiddleware);

pub struct ApiRegistry {
    apis: HashMap<Api, MetaIoHandler<Metadata, Middlewares>>,
    default_api_quota: Quota,
    custom_api_quotas: HashMap<String, Quota>,
}

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

impl ApiRegistry {
    pub fn new(api_config: ApiQuotaConfiguration) -> ApiRegistry {
        let default_api_quota: QuotaWrapper = api_config.default_global_api_quota.into();
        let custom_api_quotas: HashMap<_, Quota> = api_config
            .custom_global_api_quota
            .into_iter()
            .map(|(k, v)| (k, Into::<QuotaWrapper>::into(v).0))
            .collect();
        Self {
            apis: Default::default(),
            default_api_quota: default_api_quota.0,
            custom_api_quotas,
        }
    }

    pub fn register<F>(&mut self, api_type: Api, apis: F)
    where
        F: IntoIterator<Item = (String, RemoteProcedure<Metadata>)>,
    {
        let rate_limit_middleware =
            JsonApiRateLimitMiddleware::new(self.default_api_quota, self.custom_api_quotas.clone());
        let io_handler = self.apis.entry(api_type).or_insert_with(|| {
            MetaIoHandler::<Metadata, Middlewares>::with_middleware((
                MetricMiddleware,
                rate_limit_middleware,
            ))
        });
        io_handler.extend_with(apis);
    }

    pub fn get_apis(
        &self,
        api_types: impl IntoIterator<Item = Api>,
    ) -> MetaIoHandler<Metadata, Middlewares> {
        let rate_limit_middleware =
            JsonApiRateLimitMiddleware::new(self.default_api_quota, self.custom_api_quotas.clone());
        api_types
            .into_iter()
            .map(|api_type| self.apis.get(&api_type))
            .fold(
                MetaIoHandler::<Metadata, Middlewares>::with_middleware((
                    MetricMiddleware,
                    rate_limit_middleware,
                )),
                |mut init, apis| {
                    if let Some(apis) = apis {
                        init.extend_with(apis.iter().map(|(k, v)| (k.clone(), v.clone())));
                    }
                    init
                },
            )
    }
}
