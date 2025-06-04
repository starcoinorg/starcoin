// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::ColumnFamilyName;
use crate::{
    BLOCK_ACCUMULATOR_NODE_PREFIX_NAME, BLOCK_BODY_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME,
    BLOCK_INFO_PREFIX_NAME, BLOCK_PREFIX_NAME, BLOCK_TRANSACTIONS_PREFIX_NAME,
    BLOCK_TRANSACTION_INFOS_PREFIX_NAME, CHAIN_INFO_PREFIX_NAME, CONTRACT_EVENT_PREFIX_NAME,
    CONTRACT_EVENT_PREFIX_NAME_V2, FAILED_BLOCK_PREFIX_NAME, STATE_NODE_PREFIX_NAME,
    TABLE_INFO_PREFIX_NAME, TABLE_INFO_PREFIX_NAME_V2, TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
    TRANSACTION_INFO_HASH_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME_V2, TRANSACTION_INFO_PREFIX_NAME_V3, TRANSACTION_PREFIX_NAME,
    TRANSACTION_PREFIX_NAME_V2, VM_STATE_ACCUMULATOR_NODE_PREFIX_NAME,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use std::collections::HashSet;

///db storage use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
static VEC_PREFIX_NAME_V1: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME,
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME,
    ]
});

static VEC_PREFIX_NAME_V2: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME,
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME_V2,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME,
    ]
});

static VEC_PREFIX_NAME_V3: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME, // unused column
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME, // unused column
        TRANSACTION_INFO_PREFIX_NAME_V2,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME,
        TABLE_INFO_PREFIX_NAME,
    ]
});

static VEC_PREFIX_NAME_V4: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    let mut prefix = VEC_PREFIX_NAME_V3.iter().cloned().collect::<HashSet<_>>();
    prefix.remove(CONTRACT_EVENT_PREFIX_NAME);
    prefix.remove(TABLE_INFO_PREFIX_NAME);
    prefix.remove(TRANSACTION_PREFIX_NAME);
    prefix.remove(TRANSACTION_PREFIX_NAME);
    assert_eq!(prefix.len(), VEC_PREFIX_NAME_V3.len() - 4);

    prefix.push(CONTRACT_EVENT_PREFIX_NAME_V2);
    prefix.push(TRANSACTION_PREFIX_NAME_V2);
    prefix.push(TABLE_INFO_PREFIX_NAME_V2);
    prefix.push(TRANSACTION_INFO_PREFIX_NAME_V3);
    prefix.push(VM_STATE_ACCUMULATOR_NODE_PREFIX_NAME); // newly added

    prefix.into_iter().collect()
});

// For V4 storage, the following column families are updated from V3:
// check db_upgrade_from_v3_v4 to see the details of the upgrade.
// --------------------------------------------------------------------------------------------------
// new cf added                          | old cf to be replaced           | new key-value
// --------------------------------------------------------------------------------------------------
// VM_STATE_ACCUMULATOR_NODE_PREFIX_NAME |                                 |(HashValue,AccumulatorNode)
// CONTRACT_EVENT_PREFIX_NAME_V2         | CONTRACT_EVENT_PREFIX_NAME      |(HashValue,StcContractEvent)
// TRANSACTION_PREFIX_NAME_V2            | TRANSACTION_PREFIX_NAME         |(HashValue,StcTransaction)
// TABLE_INFO_PREFIX_NAME_V2             | TABLE_INFO_PREFIX_NAME          |(StcTableHandle,StcTableInfo)
// TRANSACTION_INFO_PREFIX_NAME_V2       | TRANSACTION_INFO_PREFIX_NAME_V3 |(HashValue,StcRichTransactionInfo)
// --------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum StorageVersion {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
}

impl StorageVersion {
    pub fn current_version() -> StorageVersion {
        StorageVersion::V4
    }

    pub fn get_column_family_names(&self) -> &'static [ColumnFamilyName] {
        match self {
            StorageVersion::V1 => &VEC_PREFIX_NAME_V1,
            StorageVersion::V2 => &VEC_PREFIX_NAME_V2,
            StorageVersion::V3 => &VEC_PREFIX_NAME_V3,
            StorageVersion::V4 => &VEC_PREFIX_NAME_V4,
        }
    }
}
