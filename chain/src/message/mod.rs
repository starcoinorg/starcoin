/// message for chain actor
use actix::prelude::*;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum ChainMessage {
    CreateBlock(u64),
}
