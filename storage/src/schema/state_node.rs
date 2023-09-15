use crate::STATE_NODE_PREFIX_NAME;
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use starcoin_state_store_api::StateNode;

define_schema!(State, HashValue, StateNode, STATE_NODE_PREFIX_NAME);

impl KeyCodec<State> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }
    fn decode_key(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec<State> for StateNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.0.clone())
    }
    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(StateNode(data.to_vec()))
    }
}
