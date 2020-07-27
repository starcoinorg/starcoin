// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::TypeTag;
use crate::parser::parse_type_tag;
use anyhow::{bail, ensure, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::StructTag;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct TokenCode {
    //Token module's address
    pub address: AccountAddress,
    //Token module's name
    pub module: String,
    //Token's struct name
    pub name: String,
}

impl TokenCode {
    pub fn new(address: AccountAddress, module: String, name: String) -> TokenCode {
        debug_assert_eq!(module, name, "Token's module name should equals name");
        Self {
            address,
            module,
            name,
        }
    }
}

impl fmt::Display for TokenCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)?;
        write!(f, "::{}", self.module)?;
        write!(f, "::{}", self.name)
    }
}

impl TryFrom<TypeTag> for TokenCode {
    type Error = anyhow::Error;

    fn try_from(value: TypeTag) -> Result<Self, Self::Error> {
        match value {
            TypeTag::Struct(struct_tag) => {
                ensure!(
                    struct_tag.type_params.is_empty(),
                    "Token's type tag should not contains type_params."
                );
                Ok(Self::new(
                    struct_tag.address,
                    struct_tag.module.into_string(),
                    struct_tag.name.into_string(),
                ))
            }
            type_tag => bail!("{:?} is not a Token's type tag", type_tag),
        }
    }
}

impl FromStr for TokenCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        Self::try_from(type_tag)
    }
}

impl Into<TypeTag> for TokenCode {
    fn into(self) -> TypeTag {
        TypeTag::Struct(StructTag {
            address: self.address,
            module: Identifier::new(self.module)
                .expect("TokenCode's module should been Identifier"),
            name: Identifier::new(self.name).expect("TokenCode's name should been Identifier"),
            type_params: vec![],
        })
    }
}
