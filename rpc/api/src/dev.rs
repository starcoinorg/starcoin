use jsonrpc_derive::rpc;

pub use self::gen_client::Client as DevClient;
use crate::types::{AnnotatedMoveValue, ContractCall};
use crate::FutureResult;
use starcoin_vm_types::{
    transaction::{SignedUserTransaction, TransactionOutput},
    vm_status::VMStatus,
};

/// TODO: remove this api, and merge it into contract.
#[rpc]
pub trait DevApi {
    /// Return the private key as bytes for `address`
    /// TODO: stablize the api.
    #[rpc(name = "dev.dryrun")]
    fn dry_run(&self, txn: SignedUserTransaction) -> FutureResult<(VMStatus, TransactionOutput)>;

    /// Call a move contract, return returned move values in lcs bytes.
    /// use contract.call instead, will remove it on next release"
    #[rpc(name = "dev.call_contract")]
    fn call_contract(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValue>>;
}
