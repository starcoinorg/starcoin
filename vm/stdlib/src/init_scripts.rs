// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction_scripts::CompiledBytes;
use crate::{CHAIN_NETWORK_STDLIB_VERSIONS, COMPILED_MOVE_CODE_DIR, INIT_SCRIPTS};
use anyhow::anyhow;
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::on_chain_config::SCRIPT_HASH_LENGTH;
use std::collections::HashMap;
use std::{fmt, path::PathBuf};

static COMPILED_INIT_SCRIPTS: Lazy<HashMap<(StdlibVersion, InitScript), CompiledBytes>> =
    Lazy::new(|| {
        let mut map = HashMap::new();
        for version in &*CHAIN_NETWORK_STDLIB_VERSIONS {
            let sub_dir = format!("{}/{}", version.as_string(), INIT_SCRIPTS);
            for script in InitScript::all() {
                let mut path = PathBuf::from(sub_dir.as_str());
                path.push(script.name());
                path.set_extension("mv");
                let code_file = COMPILED_MOVE_CODE_DIR.get_file(path);
                if code_file.is_none() {
                    continue;
                } // file doesn't exist, skip
                let compiled_bytes = CompiledBytes(code_file.unwrap().contents().to_vec());
                map.insert((*version, script), compiled_bytes);
            }
        }
        map
    });

static INIT_SCRIPTS_LIST: Lazy<HashMap<StdlibVersion, Vec<InitScript>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for version in &*CHAIN_NETWORK_STDLIB_VERSIONS {
        let sub_dir = format!("{}/{}", version.as_string(), INIT_SCRIPTS);
        let mut scripts = Vec::new();
        for script in InitScript::all() {
            let mut path = PathBuf::from(sub_dir.as_str());
            path.push(script.name());
            path.set_extension("mv");
            let code_file = COMPILED_MOVE_CODE_DIR.get_file(path);
            if code_file.is_some() {
                scripts.push(script)
            }
        }
        map.insert(*version, scripts);
    }
    map
});

pub fn compiled_init_script(version: StdlibVersion, script: InitScript) -> CompiledBytes {
    COMPILED_INIT_SCRIPTS
        .get(&(version, script))
        .expect("compiled script should exist")
        .clone()
}

pub struct VersionedInitScript {
    version: StdlibVersion,
    scripts: Vec<InitScript>,
}

impl VersionedInitScript {
    pub fn new(version: StdlibVersion) -> Self {
        VersionedInitScript {
            version,
            scripts: INIT_SCRIPTS_LIST
                .get(&version)
                .expect("init script list should exist")
                .clone(),
        }
    }
    pub fn all(&self) -> &Vec<InitScript> {
        &self.scripts
    }

    pub fn whitelist(&self) -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
        InitScript::all()
            .iter()
            .map(|script| {
                *COMPILED_INIT_SCRIPTS
                    .get(&(self.version, *script))
                    .expect("compiled script should exist")
                    .clone()
                    .hash()
                    .as_ref()
            })
            .collect()
    }

    pub fn compiled_bytes(&self, script: InitScript) -> CompiledBytes {
        COMPILED_INIT_SCRIPTS
            .get(&(self.version, script))
            .expect("compiled init script should exist")
            .clone()
    }

    /// Return the sha3-256 hash of the compiled script bytes
    pub fn hash(&self, script: &InitScript) -> HashValue {
        self.compiled_bytes(*script).hash()
    }

    pub fn is_one_of(&self, code_bytes: &[u8]) -> bool {
        let hash = CompiledBytes::hash_bytes(code_bytes);
        self.all()
            .clone()
            .iter()
            .find(|script| self.hash(*script) == hash)
            .cloned()
            .ok_or_else(|| anyhow!("Could not create standard library script from bytes"))
            .is_ok()
    }
}
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum InitScript {
    GenesisInit,
    // ...add new scripts here
}

impl InitScript {
    /// Return a vector containing all of the init scripts
    pub fn all() -> Vec<Self> {
        use InitScript::*;
        vec![
            GenesisInit,
            // ...add new scripts here
        ]
    }

    /// Return a lowercase-underscore style name for this script
    pub fn name(self) -> String {
        self.to_string()
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
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_file_correspondence() {
        // make sure that every file under transaction_scripts/ is represented in
        // InitScript::all() (and vice versa)
        for version in &*CHAIN_NETWORK_STDLIB_VERSIONS {
            let sub_dir = format!("{}/{}", version.as_string(), INIT_SCRIPTS);
            let files = COMPILED_MOVE_CODE_DIR
                .get_dir(Path::new(sub_dir.as_str()))
                .unwrap()
                .files();
            let scripts = VersionedInitScript::new(*version);

            for file in files {
                assert!(
                    scripts.is_one_of(file.contents()),
                    "File {} missing from InitScript enum",
                    file.path().display()
                )
            }
            assert_eq!(
                files.len(),
                scripts.all().len(),
                "Mismatch between stdlib script files and InitScript enum. {}",
                if files.len() > scripts.all().len() {
                    "Did you forget to extend the InitScript enum?"
                } else {
                    "Did you forget to rebuild the standard library?"
                }
            );
        }
    }
}
