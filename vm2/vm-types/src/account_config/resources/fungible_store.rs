// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::stc_fungible_asset_derive_address;
use crate::{account_config::stc_type_tag, transaction::authenticator::AuthenticationKey};
use anyhow::{anyhow, Result};
use bcs;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

///
/// For the coin_canonical_string parameter, please refer to the following code
///
/// Using starcoin_fungible_asset as the derived address if STC,
/// otherwise use type_info::type_name<CoinType>() to generate the derived address
///
/// coin_canonical_string must be a normalized Token string, such as
/// 0x0000000000000001::starcoin_coin::STC, 0x0000000000000123::USDT::USDT etc
///
/// ```
/// inline fun create_and_return_paired_metadata_if_not_exist<CoinType>(allow_stc_creation: bool): Object<Metadata> {
///     ...
///            let metadata_object_cref =
///                 if (is_stc) {
///                     object::create_sticky_object_at_address(@starcoin_framework, @starcoin_fungible_asset)
///                 } else {
///                     object::create_named_object(
///                         &create_signer::create_signer(@starcoin_fungible_asset),
///                         *string::bytes(&type_info::type_name<CoinType>())
///                     )
///                 };
///            primary_fungible_store::create_primary_store_enabled_fungible_asset(
///                &metadata_object_cref,
///                option::none(),
///                name<CoinType>(),
///                symbol<CoinType>(),
///                decimals<CoinType>(),
///                string::utf8(b""),
///                string::utf8(b""),
///            );
///  ...
/// }
///
///
const OBJECT_FROM_SEED_ADDRESS_SCHEME: u8 = 0xFE;

pub fn create_derivd_address_by_seed(
    source: &AccountAddress,
    seed: &str,
) -> Result<AccountAddress> {
    let mut bytes = bcs::to_bytes(source)?;
    bytes.extend_from_slice(seed.as_bytes());
    bytes.push(OBJECT_FROM_SEED_ADDRESS_SCHEME);
    let hash = Sha3_256::digest(&bytes);
    let truncation_hash_16 = &hash[16..32];
    Ok(AccountAddress::from_bytes(truncation_hash_16)?)
}

pub fn primary_store(
    source: &AccountAddress,
    coin_canonical_string: &str,
) -> Result<AccountAddress> {
    if coin_canonical_string.is_empty() {
        return Err(anyhow!("coin_canonical_string is empty"));
    }
    let ret_address = if coin_canonical_string == stc_type_tag().to_canonical_string() {
        AuthenticationKey::object_address_from_object(source, &stc_fungible_asset_derive_address())
            .derived_address()
    } else {
        AuthenticationKey::object_address_from_object(
            source,
            &create_derivd_address_by_seed(source, coin_canonical_string)?,
        )
        .derived_address()
    };
    Ok(ret_address)
}

/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct FungibleStoreResource {
    metadata: AccountAddress,
    balance: u64,
    frozen: bool,
}

impl FungibleStoreResource {
    pub fn new(metadata: AccountAddress, balance: u64, frozen: bool) -> Self {
        Self {
            metadata,
            balance,
            frozen,
        }
    }

    pub fn metadata(&self) -> AccountAddress {
        self.metadata
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }
}

impl MoveStructType for FungibleStoreResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("fungible_asset");
    const STRUCT_NAME: &'static IdentStr = ident_str!("FungibleStore");
}

impl MoveResource for FungibleStoreResource {}

#[derive(Debug, Serialize, Deserialize)]
struct AggregatorResource {
    value: u64,
    max_value: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConcurrentFungibleBalanceResource {
    balance: AggregatorResource,
}

impl ConcurrentFungibleBalanceResource {
    pub fn balance(&self) -> u64 {
        self.balance.value
    }
}

impl MoveStructType for ConcurrentFungibleBalanceResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("fungible_asset");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ConcurrentFungibleBalance");
}

impl MoveResource for ConcurrentFungibleBalanceResource {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_primary_store() -> Result<()> {
        let random_addr = "0x5217516b60c33f0d859e36103940bc2c".parse::<AccountAddress>()?;
        let stc_fungible_store =
            create_derivd_address_by_seed(&random_addr, &stc_type_tag().to_canonical_string())?;
        assert!(
            stc_fungible_store == "0xb9e16bcae35a6dd8c0d63c5cdc2914f8".parse::<AccountAddress>()?
        );
        Ok(())
    }
}
