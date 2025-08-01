// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    genesis_config,
    view::{str_view::StrView, transaction_payload_view::TransactionPayloadView},
};
use bcs_ext::BCSCodec;
use move_core_types::account_address::AccountAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::transaction::{RawUserTransaction, TransactionPayload};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RawUserTransactionView {
    /// Sender's address.
    pub sender: AccountAddress,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: StrView<u64>,

    // The transaction payload in bcs_ext bytes.
    pub payload: StrView<Vec<u8>>,
    // decoded transaction payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decoded_payload: Option<TransactionPayloadView>,
    // Maximal total gas specified by wallet to spend for this transaction.
    pub max_gas_amount: StrView<u64>,
    // Maximal price can be paid per gas.
    pub gas_unit_price: StrView<u64>,
    // The token code for pay transaction gas, Default is STC token code.
    pub gas_token_code: String,
    // Expiration timestamp for this transaction. timestamp is represented
    // as u64 in seconds from Unix Epoch. If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    pub expiration_timestamp_secs: StrView<u64>,
    pub chain_id: u8,
}

impl TryFrom<RawUserTransaction> for RawUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(origin: RawUserTransaction) -> Result<Self, Self::Error> {
        Ok(Self {
            sender: origin.sender(),
            sequence_number: origin.sequence_number().into(),
            max_gas_amount: origin.max_gas_amount().into(),
            gas_unit_price: origin.gas_unit_price().into(),
            gas_token_code: origin.gas_token_code(),
            expiration_timestamp_secs: origin.expiration_timestamp_secs().into(),
            chain_id: origin.chain_id().id(),
            payload: StrView(origin.into_payload().encode()?),
            decoded_payload: None,
        })
    }
}

impl From<RawUserTransactionView> for RawUserTransaction {
    fn from(transaction_view: RawUserTransactionView) -> Self {
        Self::new(
            transaction_view.sender,
            transaction_view.sequence_number.0,
            TransactionPayload::decode(transaction_view.payload.0.as_slice()).unwrap(),
            transaction_view.max_gas_amount.0,
            transaction_view.gas_unit_price.0,
            transaction_view.expiration_timestamp_secs.0,
            genesis_config::ChainId::new(transaction_view.chain_id),
            transaction_view.gas_token_code.clone(),
        )
    }
}
