use crate::BlockChain;
use anyhow::Result;
use config::NodeConfig;
use starcoin_genesis::Genesis;
use std::sync::Arc;

pub fn gen_blockchain_for_test(config: Arc<NodeConfig>) -> Result<BlockChain> {
    let (storage, startup_info, _) =
        Genesis::init_storage(config.as_ref()).expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(config.net(), *startup_info.get_master(), storage, None)?;
    Ok(block_chain)
}
