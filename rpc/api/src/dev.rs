use jsonrpc_derive::rpc;

pub use self::gen_client::Client as DevClient;
use crate::FutureResult;
use starcoin_types::transaction::{SignedUserTransaction, TransactionOutput};

#[rpc]
pub trait DevApi {
    /// Return the private key as bytes for `address`
    #[rpc(name = "dev.dryrun")]
    fn dry_run(&self, txn: SignedUserTransaction) -> FutureResult<TransactionOutput>;
}
