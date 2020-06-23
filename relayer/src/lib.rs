mod block_relayer;

use actix::prelude::*;
use starcoin_types::{block::Block, U256};

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct BlockRelayEvent {
    pub block: Block,
    pub total_difficulty: U256,
}
