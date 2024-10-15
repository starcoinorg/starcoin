use crate::move_vm_ext::StarcoinMoveResolver;
use move_binary_format::errors::VMResult;
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::session::Session;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher, PlainCryptoHash};
use starcoin_crypto::HashValue;
use starcoin_vm_runtime_types::{
    change_set::VMChangeSet, storage::change_set_configs::ChangeSetConfigs,
};
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::on_chain_config::Features;
use starcoin_vm_types::transaction::SignatureCheckedTransaction;
use starcoin_vm_types::transaction_metadata::TransactionMetadata;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

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

pub struct SessionExt<'r, 'l> {
    inner: Session<'r, 'l>,
    remote: &'r dyn StarcoinMoveResolver,
    features: Arc<Features>,
}

impl<'r, 'l> SessionExt<'r, 'l> {
    pub fn new(
        inner: Session<'r, 'l>,
        remote: &'r dyn StarcoinMoveResolver,
        features: Arc<Features>,
    ) -> Self {
        Self {
            inner,
            remote,
            features,
        }
    }

    pub fn finish(self, configs: &ChangeSetConfigs) -> VMResult<VMChangeSet> {
        // XXX FIXME YSG
        let change_set = VMChangeSet::empty();
        Ok(change_set)
    }

    pub fn into_inner(self) -> Session<'r, 'l> {
        self.inner
    }
}

impl<'r, 'l> Deref for SessionExt<'r, 'l> {
    type Target = Session<'r, 'l>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'r, 'l> DerefMut for SessionExt<'r, 'l> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// TODO(Simon): remove following code

use crate::access_path_cache::AccessPathCache;
use move_core_types::effects::{ChangeSet as MoveChangeSet, Op as MoveStorageOp};
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_table_extension::TableChangeSet;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_value::StateValueMetadata;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use std::collections::BTreeMap;

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
                let state_key = StateKey::resource(&addr, &struct_tag).unwrap();
                let ap = ap_cache.get_resource_path(addr, struct_tag);
                let op = match blob_opt {
                    MoveStorageOp::Delete => WriteOp::Deletion {
                        metadata: StateValueMetadata::none(),
                    },
                    MoveStorageOp::New(data) => WriteOp::Creation {
                        data,
                        metadata: StateValueMetadata::none(),
                    },
                    MoveStorageOp::Modify(data) => WriteOp::Modification {
                        data,
                        metadata: StateValueMetadata::none(),
                    },
                };
                write_set_mut.insert((state_key, op))
            }

            // XXX FIXME YSG check write_set need upgrade? why aptos no need MoveStorageOp
            for (name, blob_opt) in modules {
                let state_key = StateKey::module(&addr, &name);
                let ap = ap_cache.get_module_path(ModuleId::new(addr, name));
                let op = match blob_opt {
                    MoveStorageOp::Delete => WriteOp::Deletion {
                        metadata: StateValueMetadata::none(),
                    },
                    MoveStorageOp::New(data) => WriteOp::Creation {
                        data,
                        metadata: StateValueMetadata::none(),
                    },
                    MoveStorageOp::Modify(data) => WriteOp::Modification {
                        data,
                        metadata: StateValueMetadata::none(),
                    },
                };

                write_set_mut.insert((state_key, op))
            }
        }

        for (handle, change) in table_change_set.changes {
            for (key, value_op) in change.entries {
                let state_key = StateKey::table_item(&handle.into(), &key);
                // XXX FIXME YSG check write_set need upgrade? why aptos no need MoveStorageOp
                match value_op {
                    MoveStorageOp::Delete => write_set_mut.insert((
                        state_key,
                        WriteOp::Deletion {
                            metadata: StateValueMetadata::none(),
                        },
                    )),
                    MoveStorageOp::New(data) => write_set_mut.insert((
                        state_key,
                        WriteOp::Creation {
                            data,
                            metadata: StateValueMetadata::none(),
                        },
                    )),
                    MoveStorageOp::Modify(data) => write_set_mut.insert((
                        state_key,
                        WriteOp::Modification {
                            data,
                            metadata: StateValueMetadata::none(),
                        },
                    )),
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

        Ok((table_infos, write_set, vec![]))
    }
}
