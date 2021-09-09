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
    pub fn new(code: Vec<u8>) -> Module {
        Module { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
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
