// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Rust representation of a Move transaction script that can be executed on the Libra blockchain.
//! Libra does not allow arbitrary transaction scripts; only scripts whose hashes are present in
//! the on-chain script whitelist. The genesis whitelist is derived from this file, and the
//! `Stdlib` script enum will be modified to reflect changes in the on-chain whitelist as time goes
//! on.

use crate::{CHAIN_NETWORK_STDLIB_VERSIONS, COMPILED_MOVE_CODE_DIR, TRANSACTION_SCRIPTS};
use anyhow::{anyhow, bail};
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::on_chain_config::SCRIPT_HASH_LENGTH;
use std::collections::HashMap;
use std::str::FromStr;
use std::{fmt, path::PathBuf};

static COMPILED_TRANSACTION_SCRIPTS: Lazy<HashMap<(StdlibVersion, StdlibScript), CompiledBytes>> =
    Lazy::new(|| {
        let mut map = HashMap::new();
        for version in &*CHAIN_NETWORK_STDLIB_VERSIONS {
            let sub_dir = format!("{}/{}", version.as_string(), TRANSACTION_SCRIPTS);
            for script in StdlibScript::all() {
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

static TRANSACTION_SCRIPTS_LIST: Lazy<HashMap<StdlibVersion, Vec<StdlibScript>>> =
    Lazy::new(|| {
        let mut map = HashMap::new();
        for version in &*CHAIN_NETWORK_STDLIB_VERSIONS {
            let sub_dir = format!("{}/{}", version.as_string(), TRANSACTION_SCRIPTS);
            let mut scripts = Vec::new();
            for script in StdlibScript::all() {
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

pub fn compiled_transaction_script(version: StdlibVersion, script: StdlibScript) -> CompiledBytes {
    COMPILED_TRANSACTION_SCRIPTS
        .get(&(version, script))
        .expect("compiled script should exist")
        .clone()
}

pub struct VersionedStdlibScript {
    version: StdlibVersion,
    scripts: Vec<StdlibScript>,
}

impl VersionedStdlibScript {
    pub fn new(version: StdlibVersion) -> Self {
        VersionedStdlibScript {
            version,
            scripts: TRANSACTION_SCRIPTS_LIST
                .get(&version)
                .expect("script list should exist")
                .clone(),
        }
    }
    pub fn all(&self) -> &Vec<StdlibScript> {
        &self.scripts
    }

    pub fn whitelist(&self) -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
        StdlibScript::all()
            .iter()
            .map(|script| {
                *COMPILED_TRANSACTION_SCRIPTS
                    .get(&(self.version, *script))
                    .expect("compiled script should exist")
                    .clone()
                    .hash()
                    .as_ref()
            })
            .collect()
    }

    pub fn compiled_bytes(&self, script: StdlibScript) -> CompiledBytes {
        COMPILED_TRANSACTION_SCRIPTS
            .get(&(self.version, script))
            .expect("compiled script should exist")
            .clone()
    }

    /// Return the sha3-256 hash of the compiled script bytes
    pub fn hash(&self, script: &StdlibScript) -> HashValue {
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
/// All of the Move transaction scripts that can be executed on the Libra blockchain
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum StdlibScript {
    AcceptToken,
    CreateAccount,
    EmptyScript,
    PeerToPeer,
    PeerToPeerWithMetadata,
    PublishSharedEd2551PublicKey,
    // ...add new scripts here
}

impl StdlibScript {
    /// Return a vector containing all of the standard library scripts (i.e., all inhabitants of the
    /// StdlibScript enum)
    pub fn all() -> Vec<Self> {
        use StdlibScript::*;
        vec![
            AcceptToken,
            CreateAccount,
            EmptyScript,
            PeerToPeer,
            PeerToPeerWithMetadata,
            PublishSharedEd2551PublicKey,
            // ...add new scripts here
        ]
    }

    /// Return a lowercase-underscore style name for this script
    pub fn name(self) -> String {
        self.to_string()
    }
}

/// Bytes produced by compiling a Move source language script into Move bytecode
#[derive(Clone)]
pub struct CompiledBytes(pub(crate) Vec<u8>);

impl CompiledBytes {
    /// Return the sha3-256 hash of the script bytes
    pub fn hash(&self) -> HashValue {
        Self::hash_bytes(&self.0)
    }

    /// Return the sha3-256 hash of the script bytes
    pub(crate) fn hash_bytes(bytes: &[u8]) -> HashValue {
        HashValue::sha3_256_of(bytes)
    }

    /// Convert this newtype wrapper into a vector of bytes
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Display for StdlibScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use StdlibScript::*;
        write!(
            f,
            "{}",
            match self {
                AcceptToken => "accept_token",
                CreateAccount => "create_account",
                EmptyScript => "empty_script",
                PeerToPeer => "peer_to_peer",
                PeerToPeerWithMetadata => "peer_to_peer_with_metadata",
                PublishSharedEd2551PublicKey => "publish_shared_ed25519_public_key",
            }
        )
    }
}

impl FromStr for StdlibScript {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        for script in Self::all() {
            if script.name().as_str() == s {
                return Ok(script);
            }
        }
        bail!("unknown script name {}", s)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_file_correspondence() {
        // make sure that every file under transaction_scripts/ is represented in
        // StdlibScript::all() (and vice versa)
        for version in &*CHAIN_NETWORK_STDLIB_VERSIONS {
            let sub_dir = format!("{}/{}", version.as_string(), TRANSACTION_SCRIPTS);
            let files = COMPILED_MOVE_CODE_DIR
                .get_dir(Path::new(sub_dir.as_str()))
                .unwrap()
                .files();
            let scripts = VersionedStdlibScript::new(*version);

            for file in files {
                assert!(
                    scripts.is_one_of(file.contents()),
                    "File {} missing from StdlibScript enum",
                    file.path().display()
                )
            }
            assert_eq!(
                files.len(),
                scripts.all().len(),
                "Mismatch between stdlib script files and StdlibScript enum. {}",
                if files.len() > scripts.all().len() {
                    "Did you forget to extend the StdlibScript enum?"
                } else {
                    "Did you forget to rebuild the standard library?"
                }
            );
        }
    }
}
