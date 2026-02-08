// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpsee::{core::RegisterMethodError, Methods};
use starcoin_config::{Api, ApiQuotaConfiguration};
use starcoin_rpc_middleware::RpcMetrics;
use std::collections::HashMap;

pub struct ApiRegistry {
    apis: HashMap<Api, Methods>,
    _quotas: ApiQuotaConfiguration,
    _metrics: Option<RpcMetrics>,
}

impl ApiRegistry {
    pub fn new(api_quotas: ApiQuotaConfiguration, metrics: Option<RpcMetrics>) -> Self {
        Self {
            apis: Default::default(),
            _quotas: api_quotas,
            _metrics: metrics,
        }
    }

    pub fn register(&mut self, api_type: Api, methods: Methods) -> Result<(), RegisterMethodError> {
        if let Some(current) = self.apis.get_mut(&api_type) {
            current.merge(methods)?;
        } else {
            self.apis.insert(api_type, methods);
        }
        Ok(())
    }

    pub fn get_apis(
        &self,
        api_types: impl IntoIterator<Item = Api>,
    ) -> Result<Methods, RegisterMethodError> {
        let mut methods = Methods::new();
        for api_type in api_types {
            if let Some(registered) = self.apis.get(&api_type) {
                methods.merge(registered.clone())?;
            }
        }
        Ok(methods)
    }
}
