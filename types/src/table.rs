// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_vm2_vm_types::state_store::table::{
    TableHandle as TableHandleV2, TableInfo as TableInfoV2,
};
use starcoin_vm_types::state_store::table::{
    TableHandle as TableHandleV1, TableInfo as TableInfoV1,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum StcTableHandle {
    V1(TableHandleV1),
    V2(TableHandleV2),
}

impl From<TableHandleV1> for StcTableHandle {
    fn from(handle: TableHandleV1) -> Self {
        StcTableHandle::V1(handle)
    }
}

impl From<TableHandleV2> for StcTableHandle {
    fn from(handle: TableHandleV2) -> Self {
        StcTableHandle::V2(handle)
    }
}

impl StcTableHandle {}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum StcTableInfo {
    V1(TableInfoV1),
    V2(TableInfoV2),
}

impl From<TableInfoV1> for StcTableInfo {
    fn from(info: TableInfoV1) -> Self {
        StcTableInfo::V1(info)
    }
}

impl From<TableInfoV2> for StcTableInfo {
    fn from(info: TableInfoV2) -> Self {
        StcTableInfo::V2(info)
    }
}

impl StcTableInfo {
    pub fn to_v1(self) -> Option<TableInfoV1> {
        match self {
            StcTableInfo::V1(info) => Some(info),
            StcTableInfo::V2(_) => None,
        }
    }

    pub fn to_v2(self) -> Option<TableInfoV2> {
        match self {
            StcTableInfo::V1(_) => None,
            StcTableInfo::V2(info) => Some(info),
        }
    }
}
