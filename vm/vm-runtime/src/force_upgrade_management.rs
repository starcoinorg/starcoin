// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_types::account::Account;
use starcoin_vm_types::genesis_config::ChainId;

/// Note: There are several critical heights here:
/// 1. Forced Upgrade Height: When the blockchain reaches this height, the forced upgrade logic will be automatically executed.
/// 2. Transaction Opening Height: Once the blockchain height exceeds this value,
///    the transaction function of the mainnet will be enabled.
///     For details, please refer to `AddressFilter::is_blacklisted`.
/// 3. Illegal STC Destruction Height: When the height exceeds this value,
///     all STC tokens in the account balances of the blacklisted accounts will be destroyed,
///     and anyone can initiate the destruction operation.
///     For the specific implementation, please refer to `StarcoinFramework::do_burn_frozen`.
///
pub const FORCE_UPGRADE_BLOCK_NUMBER: u64 = 23009355;

pub fn get_force_upgrade_block_number(chain_id: &ChainId) -> u64 {
    if chain_id.is_main() {
        FORCE_UPGRADE_BLOCK_NUMBER
    } else if chain_id.is_test() {
        50
    } else if chain_id.is_dev() {
        5
    } else if chain_id.is_halley() || chain_id.is_proxima() {
        300
    } else if chain_id.is_barnard() {
        // add 8000 + BARNARD_HARD_FORK_HEIGHT
        16081000
    } else {
        panic!("Unknown chain type");
    }
}

pub fn create_account(private_hex: &str) -> anyhow::Result<Account> {
    let bytes = hex::decode(private_hex)?;
    let private_key = Ed25519PrivateKey::try_from(&bytes[..])?;
    let public_key = Ed25519PublicKey::from(&private_key);
    Ok(Account::with_keypair(
        private_key.into(),
        public_key.into(),
        None,
    ))
}

pub fn get_force_upgrade_account(chain_id: &ChainId) -> anyhow::Result<Account> {
    if chain_id.is_main() {
        // 0x6820910808aba0dda29b486064ffc17f
        create_account("70ec43d39c812e0c0f7b7b83e22fd0c70cf136f74c29bded7379e0d9589e4485")
    } else if chain_id.is_barnard() || chain_id.is_proxima() || chain_id.is_halley() {
        // 0x0b1d07ae560c26af9bbb8264f4c7ee73
        create_account("6105e78821ace0676faf437fb40dd6892e72f01c09351298106bad2964edb007")
    } else {
        Ok(Account::new_association())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use move_core_types::account_address::AccountAddress;

    #[test]
    fn test_get_force_upgrade_account() -> anyhow::Result<()> {
        // Main TODO(BobOng): To fixed
        // assert_eq!(get_force_upgrade_account(&ChainId::new(1))?.address(), AccountAddress::from_hex_literal("0x2dd7136c13ed8051fb20147f373f6120"));
        // Barnard 251
        assert_eq!(
            *get_force_upgrade_account(&ChainId::new(251))?.address(),
            AccountAddress::from_hex_literal("0x0b1d07ae560c26af9bbb8264f4c7ee73")?
        );
        // Proxima 252
        assert_eq!(
            get_force_upgrade_account(&ChainId::new(252))?.address(),
            &AccountAddress::from_hex_literal("0x0b1d07ae560c26af9bbb8264f4c7ee73")?
        );
        // Halley 253
        assert_eq!(
            get_force_upgrade_account(&ChainId::new(253))?.address(),
            &AccountAddress::from_hex_literal("0x0b1d07ae560c26af9bbb8264f4c7ee73")?
        );
        // Dev 254
        assert_eq!(
            get_force_upgrade_account(&ChainId::new(254))?.address(),
            &AccountAddress::from_hex_literal("0xA550C18")?
        );
        // Test 255
        assert_eq!(
            get_force_upgrade_account(&ChainId::new(254))?.address(),
            &AccountAddress::from_hex_literal("0xA550C18")?
        );

        Ok(())
    }
}
