/// Generic types.
pub mod generic {
    use serde::{Deserialize, Serialize};

    /// Consensus is mostly opaque to us
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct ConsensusMessage {
        /// Message payload.
        pub data: Vec<u8>,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub enum Message {
        /// Consensus protocol message.
        Consensus(ConsensusMessage),
    }
}
