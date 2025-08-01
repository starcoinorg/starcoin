// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::Sample;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Module {
    #[serde(with = "serde_bytes")]
    #[schemars(with = "String")]
    code: Vec<u8>,
}
impl From<Module> for Vec<u8> {
    fn from(m: Module) -> Self {
        m.code
    }
}
impl Module {
    pub fn new(code: Vec<u8>) -> Self {
        Self { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.code
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Module")
            .field("code", &hex::encode(&self.code))
            .finish()
    }
}

impl Sample for Module {
    ///Sample module's source code:
    /// ```move
    /// address 0x1{
    ///     module M{
    ///     }
    /// }
    /// ```
    ///
    fn sample() -> Self {
        Self {
            code: hex::decode(
                "a11ceb0b01000000030100020702020804100000014d0000000000000000000000000000000100",
            )
            .expect("decode sample module should success"),
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModuleBundle {
    codes: Vec<Module>,
}

impl ModuleBundle {
    pub fn new(codes: Vec<Vec<u8>>) -> ModuleBundle {
        ModuleBundle {
            codes: codes.into_iter().map(Module::new).collect(),
        }
    }

    pub fn singleton(code: Vec<u8>) -> ModuleBundle {
        ModuleBundle {
            codes: vec![Module::new(code)],
        }
    }

    pub fn into_inner(self) -> Vec<Vec<u8>> {
        self.codes.into_iter().map(Module::into_inner).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Module> {
        self.codes.iter()
    }
}

impl fmt::Debug for ModuleBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleBundle")
            .field("codes", &self.codes)
            .finish()
    }
}

impl From<Module> for ModuleBundle {
    fn from(m: Module) -> ModuleBundle {
        ModuleBundle { codes: vec![m] }
    }
}

impl IntoIterator for ModuleBundle {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Module;

    fn into_iter(self) -> Self::IntoIter {
        self.codes.into_iter()
    }
}
