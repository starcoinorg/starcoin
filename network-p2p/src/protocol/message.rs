/// Generic types.
pub mod generic {
    use crypto::HashValue;
    use serde::{Deserialize, Serialize};

    /// Consensus is mostly opaque to us
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct ConsensusMessage {
        /// Message payload.
        pub data: Vec<u8>,
    }

    /// Status sent on connection.
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Status {
        /// Protocol version.
        pub version: u32,
        /// Minimum supported version.
        pub min_supported_version: u32,
        /// Genesis block hash.
        pub genesis_hash: HashValue,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub enum Message {
        /// Consensus protocol message.
        Consensus(ConsensusMessage),
        /// Status message for handshake
        Status(Status),
    }
}
