use crate::access_path_cache::AccessPathCache;
use crate::move_vm_ext::StarcoinMoveResolver;
use move_binary_format::{
    access::ModuleAccess,
    compatibility::Compatibility,
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMResult},
    normalized::Module,
    CompiledModule, IndexKind,
};
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet as MoveChangeSet, Op as MoveStorageOp},
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
    vm_status::{StatusCode, VMStatus},
};
use move_vm_runtime::move_vm_adapter::PublishModuleBundleOption;
use move_vm_runtime::{session::Session, LoadedFunction};
use move_vm_types::{gas::GasMeter, loaded_data::runtime_types::Type};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher, PlainCryptoHash};
use starcoin_crypto::HashValue;
use starcoin_vm_runtime_types::{
    change_set::VMChangeSet, storage::change_set_configs::ChangeSetConfigs,
};
use starcoin_vm_types::{
    block_metadata::BlockMetadata,
    contract_event::ContractEvent,
    on_chain_config::Features,
    state_store::state_key::StateKey,
    state_store::state_value::StateValueMetadata,
    state_store::table::{TableHandle, TableInfo},
    transaction::SignatureCheckedTransaction,
    transaction_metadata::TransactionMetadata,
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tracing::{info, warn};
use starcoin_logger::prelude::error;
use starcoin_table_natives::TableChangeSet;

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

impl<'r, 'l> SessionExt<'r, 'l> {
    ///// wrapper of Session, push signer as the first argument of function.
    //pub fn execute_entry_function(
    //    &mut self,
    //    module: &ModuleId,
    //    function_name: &IdentStr,
    //    ty_args: Vec<TypeTag>,
    //    args: Vec<impl Borrow<[u8]>>,
    //    gas_meter: &mut impl GasMeter,
    //    sender: AccountAddress,
    //) -> VMResult<SerializedReturnValues> {
    //    let (_, func, _) = self.inner.load_function(module, function_name, &ty_args)?;
    //    let final_args = Self::check_and_rearrange_args_by_signer_position(
    //        func,
    //        args.into_iter().map(|b| b.borrow().to_vec()).collect(),
    //        sender,
    //    )?;
    //    self.inner
    //        .execute_entry_function(module, function_name, ty_args, final_args, gas_meter)
    //}

    ///// wrapper of Session, push signer as the first argument of function.
    //pub fn execute_script(
    //    &mut self,
    //    script: impl Borrow<[u8]>,
    //    ty_args: Vec<TypeTag>,
    //    args: Vec<impl Borrow<[u8]>>,
    //    gas_meter: &mut impl GasMeter,
    //    sender: AccountAddress,
    //) -> VMResult<SerializedReturnValues> {
    //    let (main, _) = self.inner.load_script(script.borrow(), ty_args.clone())?;
    //    let final_args = Self::check_and_rearrange_args_by_signer_position(
    //        main,
    //        args.into_iter().map(|b| b.borrow().to_vec()).collect(),
    //        sender,
    //    )?;
    //    self.inner
    //        .execute_script(script, ty_args, final_args, gas_meter)
    //}

    pub(crate) fn check_and_rearrange_args_by_signer_position(
        func: &LoadedFunction,
        args: Vec<Vec<u8>>,
        sender: AccountAddress,
    ) -> VMResult<Vec<Vec<u8>>> {
        let has_signer = func
            .param_tys()
            .iter()
            .position(|i| matches!(i, &Type::Signer))
            .map(|pos| {
                if pos != 0 {
                    Err(
                        PartialVMError::new(StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH)
                            .with_message(format!(
                                "Expected signer arg is this first arg, but got it at {}",
                                pos + 1
                            ))
                            .finish(Location::Undefined),
                    )
                } else {
                    Ok(true)
                }
            })
            .unwrap_or(Ok(false))?;

        if has_signer {
            let signer = MoveValue::Signer(sender);
            let mut final_args = vec![signer
                .simple_serialize()
                .expect("serialize signer should success")];
            final_args.extend(args);
            Ok(final_args)
        } else {
            Ok(args)
        }
    }

    /// Publish module bundle with custom option.
    /// The code is copied from `VMRuntime::publish_module_bundle` with modification to support ModuleBundleVerifyOption.
    pub fn publish_module_bundle_with_option(
        &mut self,
        modules: Vec<Vec<u8>>,
        sender: AccountAddress,
        gas_meter: &mut impl GasMeter,
        option: PublishModuleBundleOption,
    ) -> VMResult<()> {
        let compiled_modules =
            self.verify_module_bundle(modules.clone(), sender, gas_meter, option)?;

        let mut clean_cache = false;
        // All modules verified, publish them to data cache
        for (module, blob) in compiled_modules.into_iter().zip(modules.into_iter()) {
            let republish = if self.exists_module(&module.self_id())? {
                clean_cache = true;
                true
            } else {
                false
            };
            self.publish_module_to_data_cache(&module.self_id(), blob, republish)?;
        }

        // Clear vm runtimer loader's cache to reload new modules from state cache
        if clean_cache {
            self.empty_loader_cache()?;
        }
        Ok(())
    }

    /// Verify module bundle.
    /// The code is copied from `move_vm:::VMRuntime::publish_module_bundle` with modification to support ModuleBundleVerifyOption.
    pub fn verify_module_bundle(
        &mut self,
        modules: Vec<Vec<u8>>,
        sender: AccountAddress,
        _gas_meter: &mut impl GasMeter,
        option: PublishModuleBundleOption,
    ) -> VMResult<Vec<CompiledModule>> {
        // deserialize the modules. Perform bounds check. After this indexes can be
        // used with the `[]` operator
        let compiled_modules = match modules
            .iter()
            .map(|blob| CompiledModule::deserialize(blob))
            .collect::<PartialVMResult<Vec<_>>>()
        {
            Ok(modules) => modules,
            Err(err) => {
                warn!("[VM] module deserialization failed {:?}", err);
                return Err(err.finish(Location::Undefined));
            }
        };

        // Make sure all modules' self addresses matches the transaction sender. The self address is
        // where the module will actually be published. If we did not check this, the sender could
        // publish a module under anyone's account.
        for module in &compiled_modules {
            if module.address() != &sender {
                error!("module.address() != &sender, module name: {:?}, name: {:?} sender: {:?} ", module.address(), module.name(), sender);
                return Err(verification_error(
                    StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
                    IndexKind::AddressIdentifier,
                    module.self_handle_idx().0,
                )
                .finish(Location::Undefined));
            }
        }

        // Collect ids for modules that are published together
        let mut bundle_unverified = BTreeSet::new();

        // For now, we assume that all modules can be republished, as long as the new module is
        // backward compatible with the old module.
        //
        // TODO: in the future, we may want to add restrictions on module republishing, possibly by
        // changing the bytecode format to include an `is_upgradable` flag in the CompiledModule.
        for module in &compiled_modules {
            let module_id = module.self_id();
            if self.exists_module(&module_id)? {
                if option.only_new_module {
                    warn!(
                        "[VM] module {:?} already exists. Only allow publish new modules",
                        module_id
                    );
                    return Err(PartialVMError::new(StatusCode::INVALID_MODULE_PUBLISHER)
                        .at_index(IndexKind::ModuleHandle, module.self_handle_idx().0)
                        .finish(Location::Undefined));
                }

                let old_module_ref = self.load_module(&module_id)?;
                let old_module = CompiledModule::deserialize(old_module_ref.as_ref())
                    .map_err(|e| e.finish(Location::Undefined))?;
                //todo: Remove Module, use CompiledModule directly
                if Compatibility::new(true, false)
                    .check(&Module::new(&old_module), &Module::new(&module))
                    .is_err()
                    && !option.force_publish
                {
                    return Err(PartialVMError::new(
                        StatusCode::BACKWARD_INCOMPATIBLE_MODULE_UPDATE,
                    )
                    .finish(Location::Undefined));
                }
            }

            if !bundle_unverified.insert(module_id) {
                error!("Duplicate module name: {:?}", module.self_id().name);
                return Err(PartialVMError::new(StatusCode::DUPLICATE_MODULE_NAME)
                    .finish(Location::Undefined));
            }
        }

        // Perform bytecode and loading verification. Modules must be sorted in topological order.
        self.verify_module_bundle_for_publication(&compiled_modules)?;
        Ok(compiled_modules)
    }

    pub fn verify_script_args(
        &mut self,
        script: Vec<u8>,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        sender: AccountAddress,
    ) -> VMResult<()> {
        //load the script, perform verification
        let function = self.load_script(script.borrow(), ty_args.as_slice())?;

        Self::check_script_return(ty_args.as_slice())?;

        self.check_script_signer_and_build_args(&function, ty_args, args, sender)?;

        Ok(())
    }

    pub fn verify_script_function_args(
        &mut self,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        sender: AccountAddress,
    ) -> VMResult<()> {
        let func = self.load_function(module, function_name, &ty_args)?;

        Self::check_script_return(func.ty_args())?;

        self.check_script_signer_and_build_args(&func, ty_args, args, sender)?;

        Ok(())
    }

    // TODO(simon): what's the difference between Type and TypeTag?
    //ensure the script function not return value
    pub(crate) fn check_script_return<T: std::fmt::Debug>(return_: &[T]) -> VMResult<()> {
        if !return_.is_empty() {
            Err(PartialVMError::new(StatusCode::RET_TYPE_MISMATCH_ERROR)
                .with_message(format!(
                    "Expected script function should not return value, but got {:?}",
                    return_
                ))
                .finish(Location::Undefined))
        } else {
            Ok(())
        }
    }

    fn check_script_signer_and_build_args(
        &mut self,
        func: &LoadedFunction,
        arg_tys: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        sender: AccountAddress,
    ) -> VMResult<()> {
        let final_args = Self::check_and_rearrange_args_by_signer_position(func, args, sender)?;
        let arg_tys =
            arg_tys
                .into_iter()
                .map(|tt| self.load_type(&tt))
                .try_fold(vec![], |mut acc, ty| {
                    acc.push(ty?);
                    Ok(acc)
                })?;
        let (_, _) = self
            .deserialize_args(arg_tys, final_args)
            .map_err(|e| e.finish(Location::Undefined))?;

        Ok(())
    }

    /// Clear vm runtimer loader's cache to reload new modules from state cache
    fn empty_loader_cache(&self) -> VMResult<()> {
        self.get_move_vm().mark_loader_cache_as_invalid();
        self.get_move_vm().flush_loader_cache_if_invalidated();
        Ok(())
    }
}

// TODO(Simon): remove following code

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
                    MoveStorageOp::New((data, _layout)) => write_set_mut.insert((
                        state_key,
                        WriteOp::Creation {
                            data,
                            metadata: StateValueMetadata::none(),
                        },
                    )),
                    MoveStorageOp::Modify((data, _layout)) => write_set_mut.insert((
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
