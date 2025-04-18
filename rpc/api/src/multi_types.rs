use crate::types::SignedUserTransactionView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_types::view::signed_user_transaction_view::SignedUserTransactionView as SignedUserTransactionV2View;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransactionV2;
use starcoin_vm_types::transaction::SignedUserTransaction;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum MultiSignedUserTransactionView {
    VM1(SignedUserTransactionView),
    VM2(SignedUserTransactionV2View),
}

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
                let txn: SignedUserTransactionV2 =
                    SignedUserTransactionV2::new(txn.raw_txn.into(), txn.authenticator);
                Ok(MultiSignedUserTransaction::VM2(txn))
            }
        }
    }
}
