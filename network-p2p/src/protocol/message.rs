/// Generic types.
pub mod generic {
    use serde::{Deserialize, Serialize};
    use std::borrow::Cow;

    /// Consensus is mostly opaque to us
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct FallbackMessage {
        pub protocol_name: Cow<'static, str>,
        /// Message payload.
        pub data: Vec<u8>,
    }
}
