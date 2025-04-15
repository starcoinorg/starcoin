// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{function_arg_type_view::TransactionArgumentView, str_view::StrView};
use move_core_types::transaction_argument::convert_txn_args;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Eq, PartialEq, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub enum ArgumentsView {
    HumanReadable(Vec<TransactionArgumentView>),
    BCS(Vec<StrView<Vec<u8>>>),
}

impl ArgumentsView {
    pub fn to_bcs_bytes(&self) -> Vec<Vec<u8>> {
        match self {
            Self::HumanReadable(vs) => {
                convert_txn_args(&vs.iter().map(|v| v.0.clone()).collect::<Vec<_>>())
            }
            Self::BCS(vs) => vs.iter().map(|v| v.0.clone()).collect(),
        }
    }
}

/// Be caution:
/// We only allow passing args by TransactionArgumentView to our jsonrpc.
/// Because we cannot distinguish whether `0x12341235` is an human readable address or just some bcs bytes in hex string.
impl<'de> Deserialize<'de> for ArgumentsView {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let args = <Vec<TransactionArgumentView>>::deserialize(deserializer)?;
        Ok(Self::HumanReadable(args))
    }
}

/// Only return BCS hex string when returning arguments out of jsonrpc.
impl Serialize for ArgumentsView {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            Self::HumanReadable(_vs) => {
                // transform view to bcs first.
                let data: Vec<_> = self.to_bcs_bytes().into_iter().map(StrView).collect();
                data.serialize(serializer)
            }
            Self::BCS(data) => data.serialize(serializer),
        }
    }
}
