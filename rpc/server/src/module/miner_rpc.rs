use crate::module::map_err;
use actix::Addr;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_bus::{Bus, BusActor};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::miner::MinerApi;
use starcoin_rpc_api::FutureResult;
use starcoin_types::system_events::SubmitSealEvent;

pub struct MinerRpcImpl {
    bus: Addr<BusActor>,
}

impl MinerRpcImpl {
    pub fn new(bus: Addr<BusActor>) -> Self {
        Self { bus }
    }
}

impl MinerApi for MinerRpcImpl {
    fn submit(&self, header_hash: HashValue, nonce: u64) -> FutureResult<()> {
        let bus = self.bus.clone();
        let f = async move { bus.broadcast(SubmitSealEvent { nonce, header_hash }).await }
            .map_err(map_err);
        Box::new(f.boxed().compat())
    }
}
