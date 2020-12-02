/// Generic types.
pub mod generic {
    use serde::{Deserialize, Serialize};
    use starcoin_types::startup_info::ChainInfo;
    use std::borrow::Cow;

    /// Consensus is mostly opaque to us
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct FallbackMessage {
        pub protocol_name: Cow<'static, str>,
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
        /// Tell other peer which notification protocols we support.
        pub notif_protocols: Vec<Cow<'static, str>>,
        /// Tell other peer which rpc api we support.
        pub rpc_protocols: Vec<Cow<'static, str>>,
        /// The info of the chain
        pub info: ChainInfo,
    }
}
