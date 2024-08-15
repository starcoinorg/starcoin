use crate::access_path_cache::AccessPathCache;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::{ChangeSet as MoveChangeSet, Op as MoveStorageOp};
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_table_extension::TableChangeSet;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher, PlainCryptoHash};
use starcoin_crypto::HashValue;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::transaction::SignatureCheckedTransaction;
use starcoin_vm_types::transaction_metadata::TransactionMetadata;
use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub enum SessionId {
    Txn {
        sender: AccountAddress,
        sequence_number: u64,
    },
    BlockMeta {
        id: HashValue,
    },
    Void,
}

impl SessionId {
    pub fn txn(txn: &SignatureCheckedTransaction) -> Self {
        Self::Txn {
            sender: txn.sender(),
            sequence_number: txn.sequence_number(),
        }
    }

    pub fn txn_meta(txn_data: &TransactionMetadata) -> Self {
        Self::Txn {
            sender: txn_data.sender,
            sequence_number: txn_data.sequence_number,
        }
    }

    pub fn block_meta(block_meta: &BlockMetadata) -> Self {
        Self::BlockMeta {
            id: block_meta.id(),
        }
    }

    pub fn void() -> Self {
        Self::Void
    }

    pub fn hash(&self) -> HashValue {
        match self {
            Self::BlockMeta { id } => *id,
            _ => self.crypto_hash(),
        }
    }

    pub fn as_uuid(&self) -> HashValue {
        self.hash()
    }
}

pub struct SessionOutput {
    pub change_set: MoveChangeSet,
    pub table_change_set: TableChangeSet,
}

impl SessionOutput {
    pub fn into_change_set<C: AccessPathCache>(
        self,
        ap_cache: &mut C,
    ) -> Result<
        (
            BTreeMap<TableHandle, TableInfo>,
            WriteSet,
            Vec<ContractEvent>,
        ),
        VMStatus,
    > {
        let Self {
            change_set,
            table_change_set,
        } = self;

        // XXX FIXME YSG check write_set need upgrade? why aptos no need MoveStorageOp
        let mut write_set_mut = WriteSetMut::new(Vec::new());
        for (addr, account_changeset) in change_set.into_inner() {
            let (modules, resources) = account_changeset.into_inner();
            for (struct_tag, blob_opt) in resources {
                let ap = ap_cache.get_resource_path(addr, struct_tag);
                let op = match blob_opt {
                    MoveStorageOp::Delete => WriteOp::Deletion,
                    MoveStorageOp::New(blob) => WriteOp::Value(blob.to_vec()),
                    MoveStorageOp::Modify(blob) => WriteOp::Value(blob.to_vec()),
                };
                write_set_mut.push((StateKey::AccessPath(ap), op))
            }

            // XXX FIXME YSG check write_set need upgrade? why aptos no need MoveStorageOp
            for (name, blob_opt) in modules {
                let ap = ap_cache.get_module_path(ModuleId::new(addr, name));
                let op = match blob_opt {
                    MoveStorageOp::Delete => WriteOp::Deletion,
                    MoveStorageOp::New(blob) => WriteOp::Value(blob.to_vec()),
                    MoveStorageOp::Modify(blob) => WriteOp::Value(blob.to_vec()),
                };

                write_set_mut.push((StateKey::AccessPath(ap), op))
            }
        }

        for (handle, change) in table_change_set.changes {
            for (key, value_op) in change.entries {
                let state_key = StateKey::table_item(handle.into(), key);
                // XXX FIXME YSG check write_set need upgrade? why aptos no need MoveStorageOp
                match value_op {
                    MoveStorageOp::Delete => write_set_mut.push((state_key, WriteOp::Deletion)),
                    MoveStorageOp::New(bytes) => {
                        write_set_mut.push((state_key, WriteOp::Value(bytes.to_vec())))
                    }
                    MoveStorageOp::Modify(bytes) => {
                        write_set_mut.push((state_key, WriteOp::Value(bytes.to_vec())))
                    }
                }
            }
        }

        let mut table_infos = BTreeMap::new();
        for (key, value) in table_change_set.new_tables {
            let handle = TableHandle(key.0);
            let info = TableInfo::new(value.key_type, value.value_type);

            table_infos.insert(handle, info);
        }

        let write_set = write_set_mut
            .freeze()
            .map_err(|_| VMStatus::error(StatusCode::DATA_FORMAT_ERROR, None))?;

        // TODO(simon): removed events, use empty vector to avoid compiler complains
        Ok((table_infos, write_set, vec![]))
    }
}
