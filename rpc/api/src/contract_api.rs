pub use self::gen_client::Client as ContractClient;
use crate::types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunOutputView,
    DryRunTransactionRequest, FunctionIdView, ModuleIdView, StrView, StructTagView,
};
use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_abi_decoder::DecodedMoveValue;
use starcoin_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;

#[rpc(client, server, schema)]
pub trait ContractApi {
    /// get code of module
    #[rpc(name = "contract.get_code")]
    fn get_code(&self, module_id: StrView<ModuleId>) -> FutureResult<Option<StrView<Vec<u8>>>>;

    /// get resource data of `addr`
    #[rpc(name = "contract.get_resource")]
    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> FutureResult<Option<AnnotatedMoveStructView>>;

    /// Call a move contract, return returned move values.
    #[rpc(name = "contract.call")]
    fn call(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValueView>>;

    /// Call a move contract, return move values.
    #[rpc(name = "contract.call_v2")]
    fn call_v2(&self, call: ContractCall) -> FutureResult<Vec<DecodedMoveValue>>;

    #[rpc(name = "contract.dry_run")]
    fn dry_run(&self, txn: DryRunTransactionRequest) -> FutureResult<DryRunOutputView>;

    /// Dry run RawUserTransaction, the raw_txn parameter is RawUserTransaction's hex
    #[rpc(name = "contract.dry_run_raw")]
    fn dry_run_raw(
        &self,
        raw_txn: String,
        sender_public_key: StrView<AccountPublicKey>,
    ) -> FutureResult<DryRunOutputView>;

    #[rpc(name = "contract.resolve_function")]
    fn resolve_function(&self, function_id: FunctionIdView) -> FutureResult<FunctionABI>;
    #[rpc(name = "contract.resolve_struct")]
    fn resolve_struct(&self, struct_tag: StructTagView) -> FutureResult<StructInstantiation>;
    #[rpc(name = "contract.resolve_module")]
    fn resolve_module(&self, module_id: ModuleIdView) -> FutureResult<ModuleABI>;
}
#[test]
fn test() {
    let schema = rpc_impl_ContractApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
