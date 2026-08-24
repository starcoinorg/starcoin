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

/// Immutable historical Main block whose force-upgrade transaction used the
/// original, metered execution semantics. This must not move when a future
/// Main force-upgrade height is configured.
pub const MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER: u64 = 23_009_355;

/// Main's force-upgrade block was produced with the original, metered
/// transaction semantics. Preserve those historical consensus rules when the
/// block is replayed by a node syncing from below the upgrade height.
pub fn is_legacy_main_force_upgrade(chain_id: &ChainId, block_number: u64) -> bool {
    chain_id.is_main() && block_number == MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER
}

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
    } else if chain_id.id() == 123 {
        5
    } else {
        FORCE_UPGRADE_BLOCK_NUMBER
    }
}

fn is_force_upgrade_block_with_configured_height(
    chain_id: &ChainId,
    block_number: u64,
    configured_block_number: u64,
) -> bool {
    is_legacy_main_force_upgrade(chain_id, block_number) || block_number == configured_block_number
}

/// Whether this block must execute the implicit force-upgrade transaction.
///
/// The historical Main block remains part of this set independently of the
/// currently configured force-upgrade height so that old blocks stay
/// replayable after a future upgrade is scheduled.
pub fn is_force_upgrade_block(chain_id: &ChainId, block_number: u64) -> bool {
    is_force_upgrade_block_with_configured_height(
        chain_id,
        block_number,
        get_force_upgrade_block_number(chain_id),
    )
}

/// Whether the force-upgrade transaction at this height uses the newer
/// gas-free execution path.
pub fn is_gas_free_force_upgrade(chain_id: &ChainId, block_number: u64) -> bool {
    is_force_upgrade_block(chain_id, block_number)
        && !is_legacy_main_force_upgrade(chain_id, block_number)
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
        // 0xed9ea1f3533c14e1b52d9ff6475776ba
        create_account("650a4e2222996b607bbed13e1de45ad946cd0e66167f45efaa943a58e692e280")
    } else if chain_id.is_barnard() || chain_id.is_proxima() || chain_id.is_halley() {
        // 0x0b1d07ae560c26af9bbb8264f4c7ee73
        create_account("6105e78821ace0676faf437fb40dd6892e72f01c09351298106bad2964edb007")
    } else if chain_id.id() == 123 {
        // 0xed9ea1f3533c14e1b52d9ff6475776ba
        create_account("650a4e2222996b607bbed13e1de45ad946cd0e66167f45efaa943a58e692e280")
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
        // Main 1
        assert_eq!(
            *get_force_upgrade_account(&ChainId::new(1))?.address(),
            AccountAddress::from_hex_literal("0xed9ea1f3533c14e1b52d9ff6475776ba")?
        );
        // Test 123
        assert_eq!(
            *get_force_upgrade_account(&ChainId::new(123))?.address(),
            AccountAddress::from_hex_literal("0xed9ea1f3533c14e1b52d9ff6475776ba")?
        );
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
            get_force_upgrade_account(&ChainId::new(255))?.address(),
            &AccountAddress::from_hex_literal("0xA550C18")?
        );

        Ok(())
    }

    #[test]
    fn test_main_force_upgrade_preserves_legacy_gas_semantics_only_at_history_height() {
        let main = ChainId::new(1);
        assert_eq!(MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER, 23_009_355);
        assert!(is_legacy_main_force_upgrade(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER
        ));
        assert!(!is_gas_free_force_upgrade(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER
        ));

        assert!(!is_legacy_main_force_upgrade(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER - 1
        ));
        assert!(!is_gas_free_force_upgrade(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER - 1
        ));
        assert!(!is_legacy_main_force_upgrade(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER + 1
        ));
        assert!(!is_gas_free_force_upgrade(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER + 1
        ));

        let halley = ChainId::new(253);
        let halley_force_height = get_force_upgrade_block_number(&halley);
        assert!(!is_legacy_main_force_upgrade(&halley, halley_force_height));
        assert!(is_gas_free_force_upgrade(&halley, halley_force_height));
    }

    #[test]
    fn test_main_legacy_force_upgrade_remains_dispatched_after_future_height_change() {
        let main = ChainId::new(1);
        let future_configured_height = MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER + 1_000_000;

        assert!(is_force_upgrade_block_with_configured_height(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER,
            future_configured_height,
        ));
        assert!(is_force_upgrade_block_with_configured_height(
            &main,
            future_configured_height,
            future_configured_height,
        ));
        assert!(!is_force_upgrade_block_with_configured_height(
            &main,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER + 1,
            future_configured_height,
        ));

        let halley = ChainId::new(253);
        assert!(!is_force_upgrade_block_with_configured_height(
            &halley,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER,
            future_configured_height,
        ));
    }
}
