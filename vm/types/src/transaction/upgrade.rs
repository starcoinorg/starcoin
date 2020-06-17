// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    transaction::{Module, Script},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ModuleUpgradeOp {
    /// Publish a new Module
    Publish(Module),
    /// Update a exist module
    Update(Module),
    //TODO need support remove or deprecate module?
    //Deprecate(ModuleId),
}

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
    modules: Vec<ModuleUpgradeOp>,
    scripts: Vec<InitScript>,
}

impl UpgradePackage {
    pub fn modules(&self) -> &[ModuleUpgradeOp] {
        &self.modules
    }

    pub fn scripts(&self) -> &[InitScript] {
        &self.scripts
    }

    pub fn into_inner(self) -> (Vec<ModuleUpgradeOp>, Vec<InitScript>) {
        (self.modules, self.scripts)
    }
}
