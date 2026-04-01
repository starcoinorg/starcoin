use crate::types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunOutputView,
    DryRunTransactionRequest, FunctionIdView, ModuleIdView, StrView, StructTagView,
};
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use starcoin_abi_decoder::DecodedMoveValue;
use starcoin_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;
#[rpc(client, server, namespace = "contract", namespace_separator = ".")]
pub trait ContractApi {
    /// get code of module
    #[method(name = "get_code")]
    async fn get_code(&self, module_id: StrView<ModuleId>) -> RpcResult<Option<StrView<Vec<u8>>>>;

    /// get resource data of `addr`
    #[method(name = "get_resource")]
    async fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> RpcResult<Option<AnnotatedMoveStructView>>;

    /// Call a move contract, return returned move values.
    #[method(name = "call")]
    async fn call(&self, call: ContractCall) -> RpcResult<Vec<AnnotatedMoveValueView>>;

    /// Call a move contract, return move values.
    #[method(name = "call_v2")]
    async fn call_v2(&self, call: ContractCall) -> RpcResult<Vec<DecodedMoveValue>>;
    #[method(name = "dry_run")]
    async fn dry_run(&self, txn: DryRunTransactionRequest) -> RpcResult<DryRunOutputView>;

    /// Dry run RawUserTransaction, the raw_txn parameter is RawUserTransaction's hex
    #[method(name = "dry_run_raw")]
    async fn dry_run_raw(
        &self,
        raw_txn: String,
        sender_public_key: StrView<AccountPublicKey>,
    ) -> RpcResult<DryRunOutputView>;
    #[method(name = "resolve_function")]
    async fn resolve_function(&self, function_id: FunctionIdView) -> RpcResult<FunctionABI>;
    #[method(name = "resolve_module_function_index")]
    async fn resolve_module_function_index(
        &self,
        module_id: ModuleIdView,
        function_index: u16,
    ) -> RpcResult<FunctionABI>;
    #[method(name = "resolve_struct")]
    async fn resolve_struct(&self, struct_tag: StructTagView) -> RpcResult<StructInstantiation>;
    #[method(name = "resolve_module")]
    async fn resolve_module(&self, module_id: ModuleIdView) -> RpcResult<ModuleABI>;
}

pub use ContractApiClient as ContractApiRpcClient;
pub use ContractApiServer as ContractApiRpcServer;

/// Build jsonrpsee methods from legacy `ContractApi`.
pub fn contract_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: ContractApiServer + Send + Sync + 'static,
{
    Ok(ContractApiServer::into_rpc(api).into())
}
