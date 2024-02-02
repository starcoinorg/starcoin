use crate::storage::{CodecKVStore, ValueCodec};
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

impl LegacyTransactionStorage {
    pub fn get_transaction(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Option<LegacyTransaction>> {
        self.get(txn_hash)
    }

    pub fn save_transaction(&self, txn_info: LegacyTransaction) -> anyhow::Result<()> {
        self.put(txn_info.id(), txn_info)
    }
}
