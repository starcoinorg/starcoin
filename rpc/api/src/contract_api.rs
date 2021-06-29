use jsonrpc_derive::rpc;

pub use self::gen_client::Client as ContractClient;
use crate::types::{
    AnnotatedMoveStructView, AnnotatedMoveValueView, ContractCall, DryRunTransactionRequest,
    StrView, TransactionOutputView,
};
use crate::FutureResult;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::transaction::authenticator::AccountPublicKey;

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

    /// Dry run RawUserTransaction, the raw_txn parameter is RawUserTransaction's hex
    #[rpc(name = "contract.dry_run_raw")]
    fn dry_run_raw(
        &self,
        raw_txn: String,
        sender_public_key: StrView<AccountPublicKey>,
    ) -> FutureResult<TransactionOutputView>;
}
