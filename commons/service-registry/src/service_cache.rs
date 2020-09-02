// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::BusService;
use crate::{ActorService, RegistryAsyncService, RegistryService, ServiceRef};
use anyhow::Result;
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

    pub fn bus_ref(&mut self) -> &ServiceRef<BusService> {
        self.service_ref::<BusService>()
            .expect("BusService should exist in registry.")
    }

    pub fn service_ref<S>(&mut self) -> Result<&ServiceRef<S>>
    where
        S: ActorService,
    {
        let type_id = TypeId::of::<S>();
        let entry = self.service_ref_cache.entry(type_id);
        let any_box = match entry {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let service_ref = self.registry.service_ref_sync::<S>()?;
                e.insert(Box::new(service_ref))
            }
        };
        Ok(any_box
            .downcast_ref::<ServiceRef<S>>()
            .expect("Downcast service ref should success."))
    }
}
