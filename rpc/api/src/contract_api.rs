use jsonrpc_derive::rpc;

pub use self::gen_client::Client as ContractClient;
use crate::types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunTransactionRequest,
    StrView, TransactionOutputView,
};
use crate::FutureResult;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};

#[rpc]
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

    #[rpc(name = "contract.dry_run")]
    fn dry_run(&self, txn: DryRunTransactionRequest) -> FutureResult<TransactionOutputView>;
}
