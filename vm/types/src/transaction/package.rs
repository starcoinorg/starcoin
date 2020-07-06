// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access::ModuleAccess,
    account_address::AccountAddress,
    file_format::CompiledModule,
    transaction::{Module, Script},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct Package {
    ///Package's all Module must at same address.
    package_address: AccountAddress,
    modules: Vec<Module>,
    init_script: Option<Script>,
}

impl Package {
    pub fn new(modules: Vec<Module>, init_script: Option<Script>) -> Result<Self> {
        ensure!(!modules.is_empty(), "must at latest one module");
        let package_address = Self::parse_module_address(&modules[0])?;
        for m in &modules[1..] {
            let module_address = Self::parse_module_address(&m)?;
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
        let compiled_module = CompiledModule::deserialize(module.code())?;
        Ok(*compiled_module.address())
    }

    pub fn set_init_script(&mut self, script: Script) {
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

    pub fn init_script(&self) -> Option<&Script> {
        self.init_script.as_ref()
    }

    pub fn into_inner(self) -> (AccountAddress, Vec<Module>, Option<Script>) {
        (self.package_address, self.modules, self.init_script)
    }
}
