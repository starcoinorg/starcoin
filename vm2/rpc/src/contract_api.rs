use jsonrpc_core::{BoxFuture, Error};

pub type FutureResult<T> = BoxFuture<Result<T, Error>>;
use openrpc_derive::openrpc;
use starcoin_vm2_abi_decoder::DecodedMoveValue;
use starcoin_vm2_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_vm2_types::view::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunOutputView,
    DryRunTransactionRequest, FunctionIdView, ModuleIdView, StrView, StructTagView,
};
use starcoin_vm2_vm_types::account_address::AccountAddress;
use starcoin_vm2_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm2_vm_types::transaction::authenticator::AccountPublicKey;
// copy from https://github.com/starcoinorg/starcoin/blob/bf5ec6e44a242e9dff5ac177c1565c64c6e4b0d0/rpc/api/src/contract_api.rs#L14
#[openrpc]
pub trait ContractApi {
    /// get code of module
    #[rpc(name = "contract2.get_code")]
    fn get_code(&self, module_id: StrView<ModuleId>) -> FutureResult<Option<StrView<Vec<u8>>>>;

    /// get resource data of `addr`
    #[rpc(name = "contract2.get_resource")]
    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> FutureResult<Option<AnnotatedMoveStructView>>;

    /// Call a move contract, return returned move values.
    #[rpc(name = "contract2.call")]
    fn call(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValueView>>;

    /// Call a move contract, return move values.
    #[rpc(name = "contract2.call_v2")]
    fn call_v2(&self, call: ContractCall) -> FutureResult<Vec<DecodedMoveValue>>;

    #[rpc(name = "contract2.dry_run")]
    fn dry_run(&self, txn: DryRunTransactionRequest) -> FutureResult<DryRunOutputView>;

    /// Dry run RawUserTransaction, the raw_txn parameter is RawUserTransaction's hex
    #[rpc(name = "contract2.dry_run_raw")]
    fn dry_run_raw(
        &self,
        raw_txn: String,
        sender_public_key: StrView<AccountPublicKey>,
    ) -> FutureResult<DryRunOutputView>;
    #[rpc(name = "contract2.resolve_function")]
    fn resolve_function(&self, function_id: FunctionIdView) -> FutureResult<FunctionABI>;
    #[rpc(name = "contract2.resolve_module_function_index")]
    fn resolve_module_function_index(
        &self,
        module_id: ModuleIdView,
        function_index: u16,
    ) -> FutureResult<FunctionABI>;
    #[rpc(name = "contract2.resolve_struct")]
    fn resolve_struct(&self, struct_tag: StructTagView) -> FutureResult<StructInstantiation>;
    #[rpc(name = "contract2.resolve_module")]
    fn resolve_module(&self, module_id: ModuleIdView) -> FutureResult<ModuleABI>;
}
