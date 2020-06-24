use actix::prelude::*;
use starcoin_types::cmpact_block::CompactBlock;
use starcoin_types::peer_info::PeerId;
use starcoin_types::U256;

/// Event of received compact block
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct PeerCmpctBlockEvent {
    pub peer_id: PeerId,
    pub compact_block: CompactBlock,
}

/// Message of sending compact block to network
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct NetCmpctBlockMessage {
    pub compact_block: CompactBlock,
    pub total_difficulty: U256,
}
