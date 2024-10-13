use crate::move_vm_ext::StarcoinMoveResolver;
use crate::{counters::TIMER, natives::starcoin_natives_with_builder};
use bytes::Bytes;
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, CORE_CODE_ADDRESS},
    vm_status::StatusCode,
};
use move_vm_runtime::{config::VMConfig, move_vm::MoveVM};
use once_cell::sync::Lazy;
use starcoin_framework::natives::code::PackageRegistry;
use starcoin_metrics::TimerHelper;
use starcoin_native_interface::SafeNativeBuilder;
use starcoin_vm_types::on_chain_config::OnChainConfig;
use starcoin_vm_types::state_store::state_key::StateKey;
use std::collections::HashMap;
use std::sync::RwLock;

const WARM_VM_CACHE_SIZE: usize = 8;

pub(crate) struct WarmVmCache {
    cache: RwLock<HashMap<WarmVmId, MoveVM>>,
}

static WARM_VM_CACHE: Lazy<WarmVmCache> = Lazy::new(|| WarmVmCache {
    cache: RwLock::new(HashMap::new()),
});

impl WarmVmCache {
    pub(crate) fn get_warm_vm(
        native_builder: SafeNativeBuilder,
        vm_config: VMConfig,
        resolver: &impl StarcoinMoveResolver,
    ) -> VMResult<MoveVM> {
        WARM_VM_CACHE.get(native_builder, vm_config, resolver)
    }

    fn get(
        &self,
        mut native_builder: SafeNativeBuilder,
        vm_config: VMConfig,
        resolver: &impl StarcoinMoveResolver,
    ) -> VMResult<MoveVM> {
        #[cfg(feature = "metrics")]
        let _timer = TIMER.timer_with(&["warm_vm_get"]);
        let id = {
            #[cfg(feature = "metrics")]
            let _timer = TIMER.timer_with(&["get_warm_vm_id"]);
            WarmVmId::new(&native_builder, &vm_config, resolver)?
        };

        if let Some(vm) = self.cache.read().unwrap().get(&id) {
            #[cfg(feature = "metrics")]
            let _timer = TIMER.timer_with(&["warm_vm_cache_hit"]);
            return Ok(vm.clone());
        }

        let mut cache_locked = self.cache.write().unwrap();
        if let Some(vm) = cache_locked.get(&id) {
            return Ok(vm.clone());
        }

        {
            #[cfg(feature = "metrics")]
            let _timer = TIMER.timer_with(&["warm_vm_cache_miss"]);
            let mut cache_locked = self.cache.write().unwrap();
            if let Some(vm) = cache_locked.get(&id) {
                // Another thread has loaded it
                return Ok(vm.clone());
            }

            let vm = MoveVM::new_with_config(
                starcoin_natives_with_builder(&mut native_builder),
                vm_config,
            )?;
            Self::warm_vm_up(&vm, resolver);

            // Not using LruCache because its `::get()` requires &mut self
            if cache_locked.len() >= WARM_VM_CACHE_SIZE {
                cache_locked.clear();
            }
            cache_locked.insert(id, vm.clone());
            Ok(vm)
        }
    }

    fn warm_vm_up(vm: &MoveVM, resolver: &impl StarcoinMoveResolver) {
        #[cfg(feature = "metrics")]
        let _timer = TIMER.timer_with(&["vm_warm_up"]);

        // Loading `0x1::account` and its transitive dependency into the code cache.
        //
        // This should give us a warm VM to avoid the overhead of VM cold start.
        // Result of this load could be omitted as this is a best effort approach and won't hurt if that fails.
        //
        // Loading up `0x1::account` should be sufficient as this is the most common module
        // used for prologue, epilogue and transfer functionality.
        let _ = vm.load_module(
            &ModuleId::new(CORE_CODE_ADDRESS, ident_str!("account").to_owned()),
            resolver,
        );
    }
}

#[derive(Eq, Hash, PartialEq)]
struct WarmVmId {
    natives: Bytes,
    vm_config: Bytes,
    core_packages_registry: Option<Bytes>,
}

impl WarmVmId {
    fn new(
        native_builder: &SafeNativeBuilder,
        vm_config: &VMConfig,
        resolver: &impl StarcoinMoveResolver,
    ) -> VMResult<Self> {
        let natives = {
            #[cfg(feature = "metrics")]
            let _timer = TIMER.timer_with(&["serialize_native_builder"]);
            native_builder.id_bytes()
        };
        Ok(Self {
            natives,
            vm_config: Self::vm_config_bytes(vm_config),
            core_packages_registry: Self::core_packages_id_bytes(resolver)?,
        })
    }

    fn vm_config_bytes(vm_config: &VMConfig) -> Bytes {
        #[cfg(feature = "metrics")]
        let _timer = TIMER.timer_with(&["serialize_vm_config"]);
        bcs_ext::to_bytes(vm_config)
            .expect("Failed to serialize VMConfig.")
            .into()
    }

    fn core_packages_id_bytes(resolver: &impl StarcoinMoveResolver) -> VMResult<Option<Bytes>> {
        let bytes = {
            #[cfg(feature = "metrics")]
            let _timer = TIMER.timer_with(&["fetch_pkgreg"]);
            resolver.fetch_config_bytes(&StateKey::on_chain_config::<PackageRegistry>().map_err(
                |err| {
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("failed to create StateKey: {}", err))
                        .finish(Location::Undefined)
                },
            )?)
        };

        let core_package_registry = {
            #[cfg(feature = "metrics")]
            let _timer = TIMER.timer_with(&["deserialize_pkgreg"]);
            bytes
                .as_ref()
                .map(|bytes| PackageRegistry::deserialize_into_config(bytes))
                .transpose()
                .map_err(|err| {
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("Failed to deserialize PackageRegistry: {}", err))
                        .finish(Location::Undefined)
                })?
        };

        {
            #[cfg(feature = "metrics")]
            let _timer = TIMER.timer_with(&["ensure_no_ext_deps"]);
            core_package_registry
                .as_ref()
                .map(Self::ensure_no_external_dependency)
                .transpose()?;
        }

        Ok(bytes)
    }

    fn ensure_no_external_dependency(core_package_registry: &PackageRegistry) -> VMResult<()> {
        for package in &core_package_registry.packages {
            for dep in &package.deps {
                if dep.account != CORE_CODE_ADDRESS {
                    return Err(
                        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message("External dependency found in core packages.".to_string())
                            .finish(Location::Undefined),
                    );
                }
            }
        }
        Ok(())
    }
}
