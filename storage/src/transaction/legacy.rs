use crate::storage::ValueCodec;
use crate::{define_storage, TRANSACTION_PREFIX_NAME};
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_vm_types::transaction::LegacyTransaction;

define_storage!(
    LegacyTransactionStorage,
    HashValue,
    LegacyTransaction,
    TRANSACTION_PREFIX_NAME
);

impl ValueCodec for LegacyTransaction {
    fn encode_value(&self) -> anyhow::Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> anyhow::Result<Self> {
        Self::decode(data)
    }
}
