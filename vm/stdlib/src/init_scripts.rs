// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction_scripts::CompiledBytes;
use anyhow::{anyhow, Error, Result};
use include_dir::{include_dir, Dir};
use starcoin_crypto::HashValue;
use starcoin_vm_types::on_chain_config::SCRIPT_HASH_LENGTH;
use std::{convert::TryFrom, fmt, path::PathBuf};

#[allow(dead_code)]
const STAGED_INIT_SCRIPTS_DIR: Dir = include_dir!("staged/init_scripts");

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InitScript {
    GenesisInit,
    AssociationInit,
    ConfigInit,
    MintInit,
    PreMineInit,
    STCInit,
    FeeInit,
    // ...add new scripts here
}

impl InitScript {
    /// Return a vector containing all of the init scripts
    pub fn all() -> Vec<Self> {
        use InitScript::*;
        vec![
            GenesisInit,
            AssociationInit,
            ConfigInit,
            MintInit,
            PreMineInit,
            STCInit,
            FeeInit,
            // ...add new scripts here
        ]
    }

    /// Construct the whitelist of script hashes used to determine whether a transaction script can
    /// be executed on the Libra blockchain
    pub fn whitelist() -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
        InitScript::all()
            .iter()
            .map(|script| *script.compiled_bytes().hash().as_ref())
            .collect()
    }

    /// Return a lowercase-underscore style name for this script
    pub fn name(self) -> String {
        self.to_string()
    }

    /// Return true if `code_bytes` is the bytecode of one of the standard library scripts
    pub fn is(code_bytes: &[u8]) -> bool {
        Self::try_from(code_bytes).is_ok()
    }

    /// Return the Move bytecode produced by compiling this script. This will almost always read
    /// from disk rather invoking the compiler; genesis is the only exception.
    pub fn compiled_bytes(self) -> CompiledBytes {
        // read from disk
        let mut path = PathBuf::from(self.name());
        path.set_extension("mv");
        CompiledBytes(
            STAGED_INIT_SCRIPTS_DIR
                .get_file(path)
                .unwrap()
                .contents()
                .to_vec(),
        )
    }

    /// Return the sha3-256 hash of the compiled script bytes
    pub fn hash(self) -> HashValue {
        self.compiled_bytes().hash()
    }
}

impl TryFrom<&[u8]> for InitScript {
    type Error = Error;

    /// Return `Some(<script_name>)` if  `code_bytes` is the bytecode of one of the standard library
    /// scripts, None otherwise.
    fn try_from(code_bytes: &[u8]) -> Result<Self> {
        let hash = CompiledBytes::hash_bytes(code_bytes);
        Self::all()
            .iter()
            .find(|script| script.hash() == hash)
            .cloned()
            .ok_or_else(|| anyhow!("Could not create standard library script from bytes"))
    }
}

impl fmt::Display for InitScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InitScript::*;
        write!(
            f,
            "{}",
            match self {
                GenesisInit => "genesis_init",
                AssociationInit => "association_init",
                ConfigInit => "config_init",
                MintInit => "mint_init",
                PreMineInit => "pre_mine_init",
                STCInit => "stc_init",
                FeeInit => "fee_init",
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_correspondence() {
        let files = STAGED_INIT_SCRIPTS_DIR.files();
        let scripts = InitScript::all();
        for file in files {
            assert!(
                InitScript::is(file.contents()),
                "File {} missing from StdlibScript enum",
                file.path().display()
            )
        }
        assert_eq!(
            files.len(),
            scripts.len(),
            "Mismatch between stdlib script files and InitScript enum. {}",
            if files.len() > scripts.len() {
                "Did you forget to extend the InitScript enum?"
            } else {
                "Did you forget to rebuild the standard library?"
            }
        );
    }
}
