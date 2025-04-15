use crate::types::SignedUserTransactionView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransactionV2;
use starcoin_vm_types::transaction::SignedUserTransaction;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SignedUserTransactionV2View {
    pub transaction_hash: HashValue,
}

impl TryFrom<SignedUserTransactionV2> for SignedUserTransactionV2View {
    type Error = anyhow::Error;

    fn try_from(txn: SignedUserTransactionV2) -> Result<Self, Self::Error> {
        Ok(Self {
            transaction_hash: txn.id(),
        })
    }
}

// XXX FIXME YSG
impl From<SignedUserTransactionV2View> for SignedUserTransactionV2 {
    fn from(_txn: SignedUserTransactionV2View) -> Self {
        Self::mock()
    }
}

// XXX FIXME YSG
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum MultiSignedUserTransactionView {
    VM1(SignedUserTransactionView),
    VM2(SignedUserTransactionV2View),
}

// XXX FIXME YSG
impl TryFrom<MultiSignedUserTransaction> for MultiSignedUserTransactionView {
    type Error = anyhow::Error;

    fn try_from(txn: MultiSignedUserTransaction) -> Result<Self, Self::Error> {
        match txn {
            MultiSignedUserTransaction::VM1(txn) => Ok(Self::VM1(txn.try_into()?)),
            MultiSignedUserTransaction::VM2(txn) => Ok(Self::VM2(txn.try_into()?)),
        }
    }
}

impl TryFrom<MultiSignedUserTransactionView> for MultiSignedUserTransaction {
    type Error = anyhow::Error;
    fn try_from(txn: MultiSignedUserTransactionView) -> Result<Self, Self::Error> {
        match txn {
            MultiSignedUserTransactionView::VM1(txn) => {
                let txn: SignedUserTransaction = txn.into();
                Ok(MultiSignedUserTransaction::VM1(txn))
            }
            MultiSignedUserTransactionView::VM2(txn) => {
                let txn: SignedUserTransactionV2 = txn.into();
                Ok(MultiSignedUserTransaction::VM2(txn))
            }
        }
    }
}
