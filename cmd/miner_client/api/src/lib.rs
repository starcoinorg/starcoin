use starcoin_types::system_events::{MintBlockEvent, SealEvent};
use dyn_clone::DynClone;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

pub trait Solver: Send + DynClone {
    fn solve(
        &mut self,
        event: MintBlockEvent,
        nonce_tx: UnboundedSender<SealEvent>,
        stop_rx: UnboundedReceiver<bool>,
    );
}
