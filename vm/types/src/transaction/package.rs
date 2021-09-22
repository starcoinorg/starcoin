// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::genesis_address;
use crate::transaction::ScriptFunction;
use crate::{
    access::ModuleAccess, account_address::AccountAddress, file_format::CompiledModule,
    transaction::Module,
};
use anyhow::{ensure, Result};
use bcs_ext::Sample;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use vm::errors::Location;
#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
pub struct Package {
    ///Package's all Module must at same address.
    #[schemars(with = "String")]
    package_address: AccountAddress,
    modules: Vec<Module>,
    init_script: Option<ScriptFunction>,
}

impl Package {
    pub fn new(modules: Vec<Module>, init_script: Option<ScriptFunction>) -> Result<Self> {
        ensure!(!modules.is_empty(), "must at latest one module");
        let package_address = Self::parse_module_address(&modules[0])?;
        for m in &modules[1..] {
            let module_address = Self::parse_module_address(m)?;
            Self::check_module_address(&package_address, &module_address)?;
        }
        Ok(Self {
            package_address,
            modules,
            init_script,
        })
    }

    pub fn new_with_modules(modules: Vec<Module>) -> Result<Self> {
        Self::new(modules, None)
    }

    pub fn new_with_module(module: Module) -> Result<Self> {
        Ok(Self {
            package_address: Self::parse_module_address(&module)?,
            modules: vec![module],
            init_script: None,
        })
    }

    fn parse_module_address(module: &Module) -> Result<AccountAddress> {
        let compiled_module = CompiledModule::deserialize(module.code())
            .map_err(|e| e.finish(Location::Undefined).into_vm_status())?;
        Ok(*compiled_module.address())
    }

    pub fn set_init_script(&mut self, script: ScriptFunction) {
        self.init_script = Some(script);
    }

    fn check_module_address(
        package_address: &AccountAddress,
        module_address: &AccountAddress,
    ) -> Result<()> {
        ensure!(
            package_address == module_address,
            "module's address ({:?}) not same as package module address {:?}",
            module_address,
            package_address,
        );
        Ok(())
    }

    pub fn add_module(&mut self, module: Module) -> Result<()> {
        let module_address = Self::parse_module_address(&module)?;
        Self::check_module_address(&self.package_address, &module_address)?;
        self.modules.push(module);
        Ok(())
    }

    pub fn package_address(&self) -> AccountAddress {
        self.package_address
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn init_script(&self) -> Option<&ScriptFunction> {
        self.init_script.as_ref()
    }

    pub fn into_inner(self) -> (AccountAddress, Vec<Module>, Option<ScriptFunction>) {
        (self.package_address, self.modules, self.init_script)
    }
}

impl Sample for Package {
    fn sample() -> Self {
        Self {
            package_address: genesis_address(),
            modules: vec![Module::sample()],
            init_script: None,
        }
    }
}
