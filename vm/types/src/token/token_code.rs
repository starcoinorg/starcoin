// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::TypeTag;
use crate::move_resource::MoveResource;
use crate::parser::parse_type_tag;
use crate::token::TOKEN_MODULE_NAME;
use anyhow::{bail, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::StructTag;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct TokenCode {
    ///Token module's address
    pub address: AccountAddress,
    ///Token module's name
    pub module: String,
    ///Token's struct name
    pub name: String,
}

impl MoveResource for TokenCode {
    const MODULE_NAME: &'static str = TOKEN_MODULE_NAME;
    const STRUCT_NAME: &'static str = "TokenCode";
}

impl TokenCode {
    pub fn new(address: AccountAddress, module: String, name: String) -> TokenCode {
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
            TypeTag::Struct(struct_tag) => Ok(TokenCode::from(struct_tag)),
            type_tag => bail!("{:?} is not a Token's type tag", type_tag),
        }
    }
}
impl From<StructTag> for TokenCode {
    fn from(struct_tag: StructTag) -> Self {
        let tag_str = struct_tag.to_string();
        let s: Vec<_> = tag_str.splitn(3, "::").collect();
        //this should not happen
        assert_eq!(s.len(), 3, "invalid struct tag format");
        Self::new(
            struct_tag.address,
            struct_tag.module.into_string(),
            s[2].to_string(),
        )
    }
}
impl FromStr for TokenCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        Self::try_from(type_tag)
    }
}

#[allow(clippy::from_over_into)]
impl TryInto<StructTag> for TokenCode {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StructTag, Self::Error> {
        match parse_type_tag(self.to_string().as_str())? {
            TypeTag::Struct(s) => Ok(s),
            t => bail!("expect token code to be a struct tag, but receive {}", t),
        }
    }
}

impl TryInto<TypeTag> for TokenCode {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TypeTag, Self::Error> {
        Ok(TypeTag::Struct(self.try_into()?))
    }
}

#[cfg(test)]
mod test {
    use crate::language_storage::{StructTag, TypeTag};
    use crate::parser::parse_type_tag;
    use crate::token::token_code::TokenCode;
    use std::convert::TryInto;
    use std::str::FromStr;

    #[test]
    fn test_token_code() {
        let token = "0x00000000000000000000000000000002::LiquidityToken::LiquidityToken<0x569ab535990a17ac9afd1bc57faec683::Ddd::Ddd, 0x569ab535990a17ac9afd1bc57faec683::Bot::Bot>";
        let tc = TokenCode::from_str(token).unwrap();
        let type_tag: StructTag = tc.clone().try_into().unwrap();
        assert_eq!(token.to_string(), tc.to_string());
        assert_eq!(
            parse_type_tag(token).unwrap(),
            TypeTag::Struct(type_tag.clone())
        );
        assert_eq!(tc, type_tag.try_into().unwrap());
    }
}
