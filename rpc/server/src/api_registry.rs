// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::rate_limit_middleware::JsonApiRateLimitMiddleware;
use jsonrpc_core::{MetaIoHandler, RemoteProcedure};
use starcoin_config::{Api, ApiQuotaConfiguration};
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_middleware::{MetricMiddleware, RpcMetrics};
use std::collections::HashMap;

type Middlewares = (MetricMiddleware, JsonApiRateLimitMiddleware);

pub struct ApiRegistry {
    apis: HashMap<Api, MetaIoHandler<Metadata, Middlewares>>,
    quotas: ApiQuotaConfiguration,
    metrics: Option<RpcMetrics>,
}

impl ApiRegistry {
    pub fn new(api_quotas: ApiQuotaConfiguration, metrics: Option<RpcMetrics>) -> ApiRegistry {
        Self {
            apis: Default::default(),
            quotas: api_quotas,
            metrics,
        }
    }

    pub fn register<F>(&mut self, api_type: Api, apis: F)
    where
        F: IntoIterator<Item = (String, RemoteProcedure<Metadata>)>,
    {
        let rate_limit_middleware = JsonApiRateLimitMiddleware::from_config(self.quotas.clone());
        let metrics = self.metrics.clone();
        let io_handler = self.apis.entry(api_type).or_insert_with(|| {
            MetaIoHandler::<Metadata, Middlewares>::with_middleware((
                MetricMiddleware::new(metrics),
                rate_limit_middleware,
            ))
        });
        io_handler.extend_with(apis);
    }

    pub fn get_apis(
        &self,
        api_types: impl IntoIterator<Item = Api>,
    ) -> MetaIoHandler<Metadata, Middlewares> {
        let rate_limit_middleware = JsonApiRateLimitMiddleware::from_config(self.quotas.clone());
        let metrics = self.metrics.clone();
        api_types
            .into_iter()
            .map(|api_type| self.apis.get(&api_type))
            .fold(
                MetaIoHandler::<Metadata, Middlewares>::with_middleware((
                    MetricMiddleware::new(metrics),
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
