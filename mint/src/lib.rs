use actix::{Actor, Addr, Context};
use anyhow::Result;
use chain::ChainActor;
use config::NodeConfig;
use network::NetworkActor;

pub struct MintActor {
    mint: Mint,
    chain: Addr<ChainActor>,
}

impl MintActor {
    pub fn launch(
        _node_config: &NodeConfig,
        _network: Addr<NetworkActor>,
        chain: Addr<ChainActor>,
    ) -> Result<Addr<MintActor>> {
        let mint = Mint {};
        let actor = MintActor { mint, chain };
        Ok(actor.start())
    }
}

impl Actor for MintActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Mint actor started");
    }
}

struct Mint {}

impl Mint {
    #[cfg(any(test))]
    fn mint_nil_block_for_test(&self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
