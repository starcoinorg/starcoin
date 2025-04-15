// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{byte_code_or_script_function::ByteCodeOrScriptFunction, str_view::StrView};
use move_core_types::{parser::parse_transaction_argument, u256};
use starcoin_crypto::{CryptoMaterialError, ValidCryptoMaterialStringExt};
use starcoin_vm_types::{
    access_path::AccessPath,
    language_storage::{parse_module_id, FunctionId, ModuleId, StructTag, TypeTag},
    parser::parse_type_tag,
    sign_message::SignedMessage,
    transaction::authenticator::AccountPublicKey,
    transaction_argument::TransactionArgument,
};
use std::str::FromStr;

pub type SignedMessageView = StrView<SignedMessage>;

impl FromStr for SignedMessageView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(SignedMessage::from_str(s)?))
    }
}

impl std::fmt::Display for SignedMessageView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

//TODO auto implement FromStr for StrView<T> where T:FromStr
pub type ModuleIdView = StrView<ModuleId>;
pub type TypeTagView = StrView<TypeTag>;
pub type StructTagView = StrView<StructTag>;
pub type TransactionArgumentView = StrView<TransactionArgument>;
pub type FunctionIdView = StrView<FunctionId>;

impl std::fmt::Display for FunctionIdView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for FunctionIdView {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(FunctionId::from_str(s)?))
    }
}

impl std::fmt::Display for StrView<ModuleId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for StrView<ModuleId> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_module_id(s)?))
    }
}

impl std::fmt::Display for TypeTagView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for TypeTagView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        Ok(Self(type_tag))
    }
}

impl std::fmt::Display for StrView<StructTag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for StructTagView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let type_tag = parse_type_tag(s)?;
        match type_tag {
            TypeTag::Struct(s) => Ok(Self(*s)),
            t => anyhow::bail!("expect struct tag, actual: {}", t),
        }
    }
}

impl std::fmt::Display for StrView<TransactionArgument> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for TransactionArgumentView {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arg = parse_transaction_argument(s)?;
        Ok(Self(arg))
    }
}

impl std::fmt::Display for StrView<Vec<u8>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for StrView<Vec<u8>> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s.strip_prefix("0x").unwrap_or(s))?))
    }
}

impl FromStr for StrView<AccountPublicKey> {
    type Err = CryptoMaterialError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AccountPublicKey::from_encoded_string(s).map(StrView)
    }
}

macro_rules! impl_str_view_for {
    ($($t:ty)*) => {$(
    impl std::fmt::Display for StrView<$t> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl FromStr for StrView<$t> {
        type Err = <$t as FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<$t>().map(StrView)
        }
    }
    impl From<StrView<$t>> for $t {
        fn from(view: StrView<$t>) -> $t {
            view.0
        }
    }
    )*}
}
impl_str_view_for! {u64 i64 u128 i128 u16 i16 u32 i32 u256::U256}
impl_str_view_for! {ByteCodeOrScriptFunction AccessPath}
