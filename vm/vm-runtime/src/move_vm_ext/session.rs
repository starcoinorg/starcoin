use crate::data_cache::get_resource_group_member_from_metadata;
use crate::move_vm_ext::write_op_converter::WriteOpConverter;
use crate::move_vm_ext::{resource_state_key, StarcoinMoveResolver};
use bytes::Bytes;
use move_binary_format::{
    access::ModuleAccess,
    compatibility::Compatibility,
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMResult},
    normalized::Module,
    CompiledModule, IndexKind,
};
use move_core_types::effects::{AccountChanges, Changes};
use move_core_types::language_storage::StructTag;
use move_core_types::value::MoveTypeLayout;
use move_core_types::{
    account_address::AccountAddress,
    effects::Op as MoveStorageOp,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
    vm_status::StatusCode,
};

use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::move_vm_adapter::PublishModuleBundleOption;
use move_vm_runtime::{session::Session, LoadedFunction};
use move_vm_types::value_serde::serialize_and_allow_delayed_values;
use move_vm_types::values::Value;
use move_vm_types::{gas::GasMeter, loaded_data::runtime_types::Type};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher, PlainCryptoHash};
use starcoin_crypto::HashValue;
use starcoin_framework::natives::aggregator_natives::{
    AggregatorChangeSet, AggregatorChangeV1, NativeAggregatorContext,
};
use starcoin_framework::natives::event::NativeEventContext;
use starcoin_logger::prelude::error;
use starcoin_table_natives::NativeTableContext;
use starcoin_table_natives::TableChangeSet;
use starcoin_vm_runtime_types::module_write_set::ModuleWriteSet;
use starcoin_vm_runtime_types::{
    change_set::VMChangeSet, storage::change_set_configs::ChangeSetConfigs,
};
use starcoin_vm_types::{
    block_metadata::BlockMetadata, contract_event::ContractEvent, on_chain_config::Features,
    state_store::state_key::StateKey, transaction::SignatureCheckedTransaction,
    transaction_metadata::TransactionMetadata,
};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tracing::warn;

pub(crate) enum ResourceGroupChangeSet {
    // Merged resource groups op.
    #[allow(dead_code)]
    V0(BTreeMap<StateKey, MoveStorageOp<BytesWithResourceLayout>>),
    // Granular ops to individual resources within a group.
    #[allow(dead_code)]
    V1(BTreeMap<StateKey, BTreeMap<StructTag, MoveStorageOp<BytesWithResourceLayout>>>),
}
type AccountChangeSet = AccountChanges<Bytes, BytesWithResourceLayout>;
type ChangeSet = Changes<Bytes, BytesWithResourceLayout>;
pub type BytesWithResourceLayout = (Bytes, Option<Arc<MoveTypeLayout>>);

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

#[allow(dead_code)]
pub struct SessionExt<'r, 'l> {
    inner: Session<'r, 'l>,
    #[allow(dead_code)]
    resolver: &'r dyn StarcoinMoveResolver,
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
            resolver: remote,
            features,
        }
    }

    pub fn finish(self, configs: &ChangeSetConfigs) -> VMResult<(VMChangeSet, ModuleWriteSet)> {
        let move_vm = self.inner.get_move_vm();

        let resource_converter = |value: Value,
                                  layout: MoveTypeLayout,
                                  has_aggregator_lifting: bool|
         -> PartialVMResult<BytesWithResourceLayout> {
            let serialization_result = if has_aggregator_lifting {
                // We allow serialization of native values here because we want to
                // temporarily store native values (via encoding to ensure deterministic
                // gas charging) in block storage.
                serialize_and_allow_delayed_values(&value, &layout)?
                    .map(|bytes| (bytes.into(), Some(Arc::new(layout))))
            } else {
                // Otherwise, there should be no native values so ensure
                // serialization fails here if there are any.
                value
                    .simple_serialize(&layout)
                    .map(|bytes| (bytes.into(), None))
            };
            serialization_result.ok_or_else(|| {
                PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("Error when serializing resource {}.", value))
            })
        };

        let (change_set, mut extensions) = self
            .inner
            .finish_with_extensions_with_custom_effects(&resource_converter)?;

        let (change_set, resource_group_change_set) =
            Self::split_and_merge_resource_groups(move_vm, self.resolver, change_set)
                .map_err(|e| e.finish(Location::Undefined))?;

        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        let aggregator_context: NativeAggregatorContext = extensions.remove();
        let aggregator_change_set = aggregator_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;

        let event_context: NativeEventContext = extensions.remove();
        let events = event_context.into_events();

        let woc = WriteOpConverter::new(self.resolver, false);

        let (change_set, module_write_set) = Self::convert_change_set(
            &woc,
            change_set,
            resource_group_change_set,
            events,
            table_change_set,
            aggregator_change_set,
            configs.legacy_resource_creation_as_modification(),
        )
        .map_err(|e| e.finish(Location::Undefined))?;

        Ok((change_set, module_write_set))
    }

    #[allow(dead_code)]
    fn populate_v0_resource_group_change_set(
        change_set: &mut BTreeMap<StateKey, MoveStorageOp<BytesWithResourceLayout>>,
        state_key: StateKey,
        mut source_data: BTreeMap<StructTag, Bytes>,
        resources: BTreeMap<StructTag, MoveStorageOp<BytesWithResourceLayout>>,
    ) -> PartialVMResult<()> {
        let common_error = || {
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("populate v0 resource group change set error".to_string())
        };

        let create = source_data.is_empty();

        for (struct_tag, current_op) in resources {
            match current_op {
                MoveStorageOp::Delete => {
                    source_data.remove(&struct_tag).ok_or_else(common_error)?;
                }
                MoveStorageOp::Modify((new_data, _)) => {
                    let data = source_data.get_mut(&struct_tag).ok_or_else(common_error)?;
                    *data = new_data;
                }
                MoveStorageOp::New((data, _)) => {
                    let data = source_data.insert(struct_tag, data);
                    if data.is_some() {
                        return Err(common_error());
                    }
                }
            }
        }

        let op = if source_data.is_empty() {
            MoveStorageOp::Delete
        } else if create {
            MoveStorageOp::New((
                bcs_ext::to_bytes(&source_data)
                    .map_err(|_| common_error())?
                    .into(),
                None,
            ))
        } else {
            MoveStorageOp::Modify((
                bcs_ext::to_bytes(&source_data)
                    .map_err(|_| common_error())?
                    .into(),
                None,
            ))
        };
        change_set.insert(state_key, op);
        Ok(())
    }

    /// * Separate the resource groups from the non-resource.
    /// * non-resource groups are kept as is
    /// * resource groups are merged into the correct format as deltas to the source data
    ///   * Remove resource group data from the deltas
    ///   * Attempt to read the existing resource group data or create a new empty container
    ///   * Apply the deltas to the resource group data
    /// The process for translating Move deltas of resource groups to resources is
    /// * Add -- insert element in container
    ///   * If entry exists, Unreachable
    ///   * If group exists, Modify
    ///   * If group doesn't exist, Add
    /// * Modify -- update element in container
    ///   * If group or data doesn't exist, Unreachable
    ///   * Otherwise modify
    /// * Delete -- remove element from container
    ///   * If group or data doesn't exist, Unreachable
    ///   * If elements remain, Modify
    ///   * Otherwise delete
    ///
    /// V1 Resource group change set behavior keeps ops for individual resources separate, not
    /// merging them into a single op corresponding to the whole resource group (V0).
    fn split_and_merge_resource_groups(
        runtime: &MoveVM,
        _resolver: &dyn StarcoinMoveResolver,
        change_set: ChangeSet,
    ) -> PartialVMResult<(ChangeSet, ResourceGroupChangeSet)> {
        // The use of this implies that we could theoretically call unwrap with no consequences,
        // but using unwrap means the code panics if someone can come up with an attack.
        let common_error = || {
            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                .with_message("split_and_merge_resource_groups error".to_string())
        };
        let mut change_set_filtered = ChangeSet::new();

        //let mut maybe_resource_group_cache = resolver.release_resource_group_cache().map(|v| {
        //    v.into_iter()
        //        .map(|(k, v)| (k, v.into_iter().collect::<BTreeMap<_, _>>()))
        //        .collect::<BTreeMap<_, _>>()
        //});
        //let mut resource_group_change_set = if maybe_resource_group_cache.is_some() {
        //    ResourceGroupChangeSet::V0(BTreeMap::new())
        //} else {
        //    ResourceGroupChangeSet::V1(BTreeMap::new())
        //};
        for (addr, account_changeset) in change_set.into_inner() {
            let mut resource_groups: BTreeMap<
                StructTag,
                BTreeMap<StructTag, MoveStorageOp<BytesWithResourceLayout>>,
            > = BTreeMap::new();
            let mut resources_filtered = BTreeMap::new();
            let (modules, resources) = account_changeset.into_inner();

            for (struct_tag, blob_op) in resources {
                let resource_group_tag = runtime
                    .with_module_metadata(&struct_tag.module_id(), |md| {
                        get_resource_group_member_from_metadata(&struct_tag, md)
                    });

                if let Some(resource_group_tag) = resource_group_tag {
                    if resource_groups
                        .entry(resource_group_tag)
                        .or_default()
                        .insert(struct_tag, blob_op)
                        .is_some()
                    {
                        return Err(common_error());
                    }
                } else {
                    resources_filtered.insert(struct_tag, blob_op);
                }
            }

            change_set_filtered
                .add_account_changeset(
                    addr,
                    AccountChangeSet::from_modules_resources(modules, resources_filtered),
                )
                .map_err(|_| common_error())?;

            //for (resource_group_tag, resources) in resource_groups {
            //    let state_key = StateKey::resource_group(&addr, &resource_group_tag);
            //    match &mut resource_group_change_set {
            //        ResourceGroupChangeSet::V0(v0_changes) => {
            //            let source_data = maybe_resource_group_cache
            //                .as_mut()
            //                .expect("V0 cache must be set")
            //                .remove(&state_key)
            //                .unwrap_or_default();
            //            Self::populate_v0_resource_group_change_set(
            //                v0_changes,
            //                state_key,
            //                source_data,
            //                resources,
            //            )?;
            //        }
            //        ResourceGroupChangeSet::V1(v1_changes) => {
            //            // Maintain the behavior of failing the transaction on resource
            //            // group member existence invariants.
            //            for (struct_tag, current_op) in resources.iter() {
            //                let exists =
            //                    resolver.resource_exists_in_group(&state_key, struct_tag)?;
            //                if matches!(current_op, MoveStorageOp::New(_)) == exists {
            //                    // Deletion and Modification require resource to exist,
            //                    // while creation requires the resource to not exist.
            //                    return Err(common_error());
            //                }
            //            }
            //            v1_changes.insert(state_key, resources);
            //        }
            //    }
            //}
        }

        Ok((
            change_set_filtered,
            ResourceGroupChangeSet::V0(BTreeMap::new()),
        ))
    }

    fn convert_change_set(
        woc: &WriteOpConverter,
        change_set: ChangeSet,
        _resource_group_change_set: ResourceGroupChangeSet,
        events: Vec<(ContractEvent, Option<MoveTypeLayout>)>,
        table_change_set: TableChangeSet,
        aggregator_change_set: AggregatorChangeSet,
        legacy_resource_creation_as_modification: bool,
    ) -> PartialVMResult<(VMChangeSet, ModuleWriteSet)> {
        let mut resource_write_set = BTreeMap::new();
        let resource_group_write_set = BTreeMap::new();

        let mut has_modules_published_to_special_address = false;
        let mut module_write_ops = BTreeMap::new();

        let mut aggregator_v1_write_set = BTreeMap::new();
        let mut aggregator_v1_delta_set = BTreeMap::new();

        for (addr, account_changeset) in change_set.into_inner() {
            let (modules, resources) = account_changeset.into_inner();
            for (struct_tag, blob_and_layout_op) in resources {
                let state_key = resource_state_key(&addr, &struct_tag)?;
                let op = woc.convert_resource(
                    &state_key,
                    blob_and_layout_op,
                    legacy_resource_creation_as_modification,
                )?;

                resource_write_set.insert(state_key, op);
            }

            for (name, blob_op) in modules {
                if addr.is_special() {
                    has_modules_published_to_special_address = true;
                }
                let state_key = StateKey::module(&addr, &name);
                let op = woc.convert_module(&state_key, blob_op, false)?;
                module_write_ops.insert(state_key, op);
            }
        }

        //match resource_group_change_set {
        //    ResourceGroupChangeSet::V0(v0_changes) => {
        //        for (state_key, blob_op) in v0_changes {
        //            let op = woc.convert_resource(&state_key, blob_op, false)?;
        //            resource_write_set.insert(state_key, op);
        //        }
        //    }
        //    ResourceGroupChangeSet::V1(v1_changes) => {
        //        for (state_key, resources) in v1_changes {
        //            let group_write = woc.convert_resource_group_v1(&state_key, resources)?;
        //            resource_group_write_set.insert(state_key, group_write);
        //        }
        //    }
        //}

        for (handle, change) in table_change_set.changes {
            for (key, value_op) in change.entries {
                let state_key = StateKey::table_item(&handle.into(), &key);
                let op = woc.convert_resource(&state_key, value_op, false)?;
                resource_write_set.insert(state_key, op);
            }
        }

        for (state_key, change) in aggregator_change_set.aggregator_v1_changes {
            match change {
                AggregatorChangeV1::Write(value) => {
                    let write_op = woc.convert_aggregator_modification(&state_key, value)?;
                    aggregator_v1_write_set.insert(state_key, write_op);
                }
                AggregatorChangeV1::Merge(delta_op) => {
                    aggregator_v1_delta_set.insert(state_key, delta_op);
                }
                AggregatorChangeV1::Delete => {
                    let write_op =
                        woc.convert_aggregator(&state_key, MoveStorageOp::Delete, false)?;
                    aggregator_v1_write_set.insert(state_key, write_op);
                }
            }
        }

        // We need to remove values that are already in the writes.
        let reads_needing_exchange = aggregator_change_set
            .reads_needing_exchange
            .into_iter()
            .filter(|(state_key, _)| !resource_write_set.contains_key(state_key))
            .collect();

        let group_reads_needing_change = aggregator_change_set
            .group_reads_needing_exchange
            .into_iter()
            .filter(|(state_key, _)| !resource_group_write_set.contains_key(state_key))
            .collect();

        let change_set = VMChangeSet::new_expanded(
            resource_write_set,
            resource_group_write_set,
            aggregator_v1_write_set,
            aggregator_v1_delta_set,
            aggregator_change_set.delayed_field_changes,
            reads_needing_exchange,
            group_reads_needing_change,
            events,
        )?;
        let module_write_set =
            ModuleWriteSet::new(has_modules_published_to_special_address, module_write_ops);

        Ok((change_set, module_write_set))
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
                error!(
                    "module.address() != &sender, module name: {:?}, name: {:?} sender: {:?} ",
                    module.address(),
                    module.name(),
                    sender
                );
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
                    .check(&Module::new(&old_module), &Module::new(module))
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
        // load the script, perform verification
        let function = self.load_script(script.borrow(), ty_args.as_slice())?;

        Self::check_script_return(function.return_tys())?;

        self.check_script_signer_and_build_args(
            &function,
            function.ty_args().to_vec(),
            args,
            sender,
        )?;

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
        let param_tys = func.param_tys().to_owned();

        Self::check_script_return(func.return_tys())?;

        self.check_script_signer_and_build_args(&func, param_tys, args, sender)?;

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
        arg_tys: Vec<Type>,
        args: Vec<Vec<u8>>,
        sender: AccountAddress,
    ) -> VMResult<()> {
        let final_args = Self::check_and_rearrange_args_by_signer_position(func, args, sender)?;
        // let arg_tys =
        //     arg_tys
        //         .into_iter()
        //         .map(|tt| self.load_type(&tt))
        //         .try_fold(vec![], |mut acc, ty| {
        //             acc.push(ty?);
        //             Ok(acc)
        //         })?;
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
