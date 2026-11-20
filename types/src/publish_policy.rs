// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Which code-carrying txns Main nodes accept and put in blocks.
//!
//! On Main, a node refuses scripts and packages, except packages that a listed project account
//! publishes to its own address. Calls to entry functions already on chain are unaffected, and
//! the framework at 0x1 keeps its own DAO and force-upgrade paths. This is node policy, not a
//! consensus rule: Main blocks are built only under our block permits, so a txn our nodes refuse
//! does not reach a Main block. Other networks are unaffected.

use crate::account_address::AccountAddress;
use crate::genesis_config::ChainId;
use crate::transaction::TransactionPayload;

/// Project accounts that still publish and upgrade their own packages on Main.
pub const MAIN_PACKAGE_PUBLISHERS: [AccountAddress; 2] = [
    // Native bridge (USDT).
    AccountAddress::new([
        0x88, 0xd4, 0xdb, 0x86, 0x60, 0xb9, 0xa4, 0xc5, 0xd9, 0x1b, 0x8d, 0x0b, 0x40, 0xc5, 0x78,
        0xe2,
    ]),
    // Starswap.
    AccountAddress::new([
        0x8c, 0x10, 0x93, 0x49, 0xc6, 0xbd, 0x91, 0x41, 0x1d, 0x6b, 0xc9, 0x62, 0xe0, 0x80, 0xc4,
        0xa3,
    ]),
];

/// Why a Main node refuses a txn's payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusedPayload {
    Script,
    Package(AccountAddress),
}

/// `Err` when a node on `chain_id` must neither accept nor include a txn with this payload.
pub fn check_payload(
    chain_id: ChainId,
    payload: &TransactionPayload,
) -> Result<(), RefusedPayload> {
    if !chain_id.is_main() {
        return Ok(());
    }
    match payload {
        TransactionPayload::ScriptFunction(_) => Ok(()),
        TransactionPayload::Script(_) => Err(RefusedPayload::Script),
        TransactionPayload::Package(p) => {
            let address = p.package_address();
            if MAIN_PACKAGE_PUBLISHERS.contains(&address) {
                Ok(())
            } else {
                Err(RefusedPayload::Package(address))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::Identifier;
    use crate::language_storage::ModuleId;
    use crate::transaction::{Module, Package, Script, ScriptFunction};
    use std::str::FromStr;

    fn package_at(address: AccountAddress) -> TransactionPayload {
        // A minimal module header naming `address`, enough for Package to read its address.
        let module = starcoin_vm_types::file_format::empty_module();
        let mut module = module;
        module.address_identifiers[0] = address;
        let mut code = vec![];
        module.serialize(&mut code).unwrap();
        TransactionPayload::Package(Package::new_with_module(Module::new(code)).unwrap())
    }

    #[test]
    fn main_refuses_scripts_and_unlisted_packages() {
        let main = ChainId::new(1);
        let call = TransactionPayload::ScriptFunction(ScriptFunction::new(
            ModuleId::new(AccountAddress::ONE, Identifier::new("M").unwrap()),
            Identifier::new("f").unwrap(),
            vec![],
            vec![],
        ));
        assert_eq!(check_payload(main, &call), Ok(()));
        let script = TransactionPayload::Script(Script::new(vec![1, 2, 3], vec![], vec![]));
        assert_eq!(check_payload(main, &script), Err(RefusedPayload::Script));
        let other = AccountAddress::from_str("0x2").unwrap();
        assert_eq!(
            check_payload(main, &package_at(other)),
            Err(RefusedPayload::Package(other))
        );
        for listed in MAIN_PACKAGE_PUBLISHERS {
            assert_eq!(check_payload(main, &package_at(listed)), Ok(()));
        }
        assert_eq!(
            check_payload(main, &package_at(AccountAddress::ONE)),
            Err(RefusedPayload::Package(AccountAddress::ONE))
        );
    }

    #[test]
    fn other_networks_are_unaffected() {
        let script = TransactionPayload::Script(Script::new(vec![1, 2, 3], vec![], vec![]));
        for id in [251u8, 252, 253, 254, 255] {
            assert_eq!(check_payload(ChainId::new(id), &script), Ok(()));
        }
    }

    #[test]
    fn listed_addresses_are_the_project_accounts() {
        assert_eq!(
            MAIN_PACKAGE_PUBLISHERS[0],
            AccountAddress::from_str("0x88d4db8660b9a4c5d91b8d0b40c578e2").unwrap()
        );
        assert_eq!(
            MAIN_PACKAGE_PUBLISHERS[1],
            AccountAddress::from_str("0x8c109349c6bd91411d6bc962e080c4a3").unwrap()
        );
    }
}
