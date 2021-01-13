use anyhow::{format_err, Result};
use futures::channel::mpsc::UnboundedSender;
use network::PeerEvent;

#[derive(Clone, Debug)]
pub struct PeerEventHandle {
    sender: UnboundedSender<PeerEvent>,
}

impl PeerEventHandle {
    pub fn new(sender: UnboundedSender<PeerEvent>) -> Self {
        Self { sender }
    }

    pub fn push(&mut self, event: PeerEvent) -> Result<()> {
        self.sender
            .start_send(event)
            .map_err(|e| format_err!("Send peer event failed : {:?}", e))
    }
}
