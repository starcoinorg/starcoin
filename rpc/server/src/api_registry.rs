// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::{MetaIoHandler, RemoteProcedure};
use starcoin_config::Api;
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_middleware::MetricMiddleware;
use std::collections::HashMap;

#[derive(Default)]
pub struct ApiRegistry {
    apis: HashMap<Api, MetaIoHandler<Metadata, MetricMiddleware>>,
}

impl ApiRegistry {
    pub fn register<F>(&mut self, api_type: Api, apis: F)
    where
        F: IntoIterator<Item = (String, RemoteProcedure<Metadata>)>,
    {
        let io_handler = self.apis.entry(api_type).or_insert_with(|| {
            MetaIoHandler::<Metadata, MetricMiddleware>::with_middleware(MetricMiddleware)
        });
        io_handler.extend_with(apis);
    }

    pub fn get_apis(
        &self,
        api_types: impl IntoIterator<Item = Api>,
    ) -> MetaIoHandler<Metadata, MetricMiddleware> {
        api_types
            .into_iter()
            .map(|api_type| self.apis.get(&api_type).cloned())
            .fold(
                MetaIoHandler::<Metadata, MetricMiddleware>::with_middleware(MetricMiddleware),
                |mut init, apis| {
                    if let Some(apis) = apis {
                        init.extend_with(apis);
                    }
                    init
                },
            )
    }
}
