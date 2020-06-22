// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    transaction::{Module, Script},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct InitScript {
    /// Execute the script by this account, if this is None, use the txn's sender.
    su_account: Option<AccountAddress>,
    script: Script,
}

impl InitScript {
    pub fn into_inner(self) -> (Option<AccountAddress>, Script) {
        (self.su_account, self.script)
    }

    pub fn su_account(&self) -> Option<AccountAddress> {
        self.su_account
    }

    pub fn script(&self) -> &Script {
        &self.script
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct UpgradePackage {
    modules: Vec<Module>,
    scripts: Vec<InitScript>,
}

impl UpgradePackage {
    pub fn new(modules: Vec<Module>, scripts: Vec<InitScript>) -> Self {
        Self { modules, scripts }
    }

    pub fn new_with_modules(modules: Vec<Module>) -> Self {
        Self {
            modules,
            scripts: vec![],
        }
    }

    pub fn add_scripts(&mut self, su_account: Option<AccountAddress>, script: Script) {
        self.scripts.push(InitScript { su_account, script });
    }
}

impl UpgradePackage {
    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn scripts(&self) -> &[InitScript] {
        &self.scripts
    }

    pub fn into_inner(self) -> (Vec<Module>, Vec<InitScript>) {
        (self.modules, self.scripts)
    }
}
