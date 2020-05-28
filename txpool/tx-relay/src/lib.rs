use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::TXN_PROTOCOL_NAME;
use std::borrow::Cow;

pub enum TxnRelayMessage {
    /// propagate local txns to remote peers,
    PropagateNewTransactions(PropagateNewTransactions),
    /// txns received from remote peers.
    PeerTransactions(PeerTransactions),
}

#[derive(Clone, Debug)]
pub struct PropagateNewTransactions {
    txns: Vec<SignedUserTransaction>,
    protocol_name: Cow<'static, [u8]>,
}

impl From<Vec<SignedUserTransaction>> for PropagateNewTransactions {
    fn from(txns: Vec<SignedUserTransaction>) -> Self {
        Self {
            txns,
            protocol_name: TXN_PROTOCOL_NAME.into(),
        }
    }
}

impl PropagateNewTransactions {
    pub fn transactions_to_propagate(self) -> Vec<SignedUserTransaction> {
        self.txns
    }

    pub fn protocol_name(&self) -> Cow<'static, [u8]> {
        self.protocol_name.clone()
    }
}
impl actix::Message for PropagateNewTransactions {
    type Result = ();
}

#[derive(Clone, Debug)]
pub struct PeerTransactions {
    txns: Vec<SignedUserTransaction>,
}
impl actix::Message for PeerTransactions {
    type Result = ();
}

impl PeerTransactions {
    pub fn new(txns: Vec<SignedUserTransaction>) -> Self {
        Self { txns }
    }

    pub fn peer_transactions(self) -> Vec<SignedUserTransaction> {
        self.txns
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
