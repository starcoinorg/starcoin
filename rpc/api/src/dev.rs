use jsonrpc_derive::rpc;

pub use self::gen_client::Client as DevClient;
use crate::types::{AnnotatedMoveValue, ContractCall};
use crate::FutureResult;
/// TODO(Deprecated): remove this api, and merge it into contract.
#[rpc]
pub trait DevApi {
    /// Call a move contract, return returned move values in lcs bytes.
    /// use contract.call instead, will remove it on next release"
    #[rpc(name = "dev.call_contract")]
    fn call_contract(&self, call: ContractCall) -> FutureResult<Vec<AnnotatedMoveValue>>;
}
