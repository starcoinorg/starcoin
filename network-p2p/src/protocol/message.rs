/// Generic types.
pub mod generic {
    use crypto::HashValue;
    use serde::{Deserialize, Serialize};
    use types::peer_info::PeerInfo;

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

        pub info: PeerInfo,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub enum Message {
        /// Status message for handshake
        Status(Box<Status>),
    }
}
