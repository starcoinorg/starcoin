// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::BusService;
use crate::{ActorService, RegistryAsyncService, RegistryService, ServiceRef};
use anyhow::{format_err, Result};
use futures::executor::block_on;
use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub(crate) struct ServiceCache {
    registry: ServiceRef<RegistryService>,
    service_ref_cache: HashMap<TypeId, Box<dyn Any + Send>>,
}

impl ServiceCache {
    pub fn new(registry: ServiceRef<RegistryService>) -> Self {
        Self {
            registry,
            service_ref_cache: HashMap::new(),
        }
    }

    pub fn registry_ref(&self) -> &ServiceRef<RegistryService> {
        &self.registry
    }

    pub fn bus_ref(&mut self) -> Result<&ServiceRef<BusService>> {
        self.service_ref::<BusService>().and_then(|service| {
            service.ok_or_else(|| format_err!("BusService should exist in registry."))
        })
    }

    pub fn service_ref<S>(&mut self) -> Result<Option<&ServiceRef<S>>>
    where
        S: ActorService,
    {
        let type_id = TypeId::of::<S>();
        let entry = self.service_ref_cache.entry(type_id);
        let any_box = match entry {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let registry = &self.registry;
                let service_ref = block_on(async move { registry.service_ref_opt::<S>().await })?;
                e.insert(Box::new(service_ref))
            }
        };
        Ok(any_box
            .downcast_ref::<Option<ServiceRef<S>>>()
            .expect("Downcast service ref should success.")
            .as_ref())
    }
}
