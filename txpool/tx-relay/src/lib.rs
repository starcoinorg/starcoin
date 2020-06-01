use starcoin_types::transaction::SignedUserTransaction;

pub enum TxnRelayMessage {
    /// propagate local txns to remote peers,
    PropagateNewTransactions(PropagateNewTransactions),
    /// txns received from remote peers.
    PeerTransactions(PeerTransactions),
}

#[derive(Clone, Debug)]
pub enum PropagateNewTransactions {
    V1(Vec<SignedUserTransaction>),
}

impl From<Vec<SignedUserTransaction>> for PropagateNewTransactions {
    fn from(txns: Vec<SignedUserTransaction>) -> Self {
        PropagateNewTransactions::V1(txns)
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
