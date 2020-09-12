use jsonrpc_derive::rpc;

pub use self::gen_client::Client as MinerClient;
use crate::FutureResult;
use starcoin_crypto::HashValue;

#[rpc]
pub trait MinerApi {
    /// submit mining seal
    #[rpc(name = "mining.submit")]
    fn submit(&self, header_hash: HashValue, nonce: u64) -> FutureResult<()>;
}
