// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{function_arg_type_view::ModuleIdView, ByteCode};
use move_core_types::identifier::Identifier;
use starcoin_vm_types::language_storage::FunctionId;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum ByteCodeOrScriptFunction {
    ByteCode(ByteCode),
    ScriptFunction(FunctionId),
}

impl std::fmt::Display for ByteCodeOrScriptFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::ByteCode(c) => write!(f, "0x{}", hex::encode(c)),
            Self::ScriptFunction(FunctionId { module, function }) => {
                write!(f, "{}::{}", module, function)
            }
        }
    }
}

impl FromStr for ByteCodeOrScriptFunction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits: Vec<&str> = s.rsplitn(2, "::").collect();
        if splits.len() == 2 {
            let module_id = ModuleIdView::from_str(splits[1])?;
            let function = Identifier::new(splits[0])?;
            Ok(Self::ScriptFunction(FunctionId {
                module: module_id.0,
                function,
            }))
        } else {
            Ok(Self::ByteCode(hex::decode(
                s.strip_prefix("0x").unwrap_or(s),
            )?))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::view::{byte_code_or_script_function::ByteCodeOrScriptFunction, str_view::StrView};
    use move_core_types::{account_address::AccountAddress, u256};
    use starcoin_vm_types::language_storage::FunctionId;

    #[test]
    fn test_script_data() {
        let script_function: ByteCodeOrScriptFunction = "0x1::M::func1".parse().unwrap();
        assert!(matches!(
            script_function,
            ByteCodeOrScriptFunction::ScriptFunction { .. }
        ));
        if let ByteCodeOrScriptFunction::ScriptFunction(FunctionId { module, function }) =
            script_function
        {
            assert_eq!(*module.address(), "0x1".parse::<AccountAddress>().unwrap());
            assert_eq!(module.name().as_str(), "M");
            assert_eq!(function.as_str(), "func1");
        }

        let bytecode: ByteCodeOrScriptFunction = "0x123432ab34".parse().unwrap();
        assert!(matches!(bytecode, ByteCodeOrScriptFunction::ByteCode(_)));
    }

    #[test]
    fn test_str_view_u256() {
        let str = "115792089237316195423570985008687907853269984665640564039457584007913129638801";
        let b = u256::U256::from_str_radix(str, 10).unwrap();
        let val: StrView<u256::U256> = b.into();
        let val_str = format!("{}", val);
        assert_eq!(str, val_str.as_str());
    }
}
