//! Fixtures shared by the fuzz targets and the seed corpus test.

use anyhow::Result;
use starcoin_vm_types::access_path::{AccessPath, DataPath};
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::StateView;
use std::collections::BTreeMap;

/// Serves the latest stdlib modules as chain state, so resolution reaches real structs.
pub struct StdlibStateView {
    modules: BTreeMap<ModuleId, Vec<u8>>,
}

impl StdlibStateView {
    pub fn load() -> Self {
        let modules = stdlib::load_latest_compiled_modules()
            .into_iter()
            .map(|m: CompiledModule| {
                let mut code = vec![];
                m.serialize(&mut code).expect("stdlib module serializes");
                (m.self_id(), code)
            })
            .collect();
        Self { modules }
    }
}

impl StateView for StdlibStateView {
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        Ok(match state_key {
            StateKey::AccessPath(AccessPath {
                address,
                path: DataPath::Code(name),
            }) => self
                .modules
                .get(&ModuleId::new(*address, name.clone()))
                .cloned(),
            _ => None,
        })
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
