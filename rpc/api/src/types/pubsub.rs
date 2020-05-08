use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeader;

/// Subscription kind.
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// New block headers subscription.
    NewHeads,
    /// New Pending Transactions subscription.
    NewPendingTransactions,
}

/// Subscription result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
    /// New block header.
    Header(Box<BlockHeader>),
    /// Transaction hash
    TransactionHash(Vec<HashValue>),
}
impl Serialize for Result {
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Result::Header(ref header) => header.serialize(serializer),
            // Result::Log(ref log) => log.serialize(serializer),
            Result::TransactionHash(ref hash) => hash.serialize(serializer),
            // Result::SyncState(ref sync) => sync.serialize(serializer),
        }
    }
}

/// Subscription kind.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Params {
    /// No parameters passed.
    None,
    // /// Log parameters.
    // Logs(Filter),
}

impl Default for Params {
    fn default() -> Self {
        Params::None
    }
}

impl<'a> Deserialize<'a> for Params {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Params, D::Error>
    where
        D: Deserializer<'a>,
    {
        let v: Value = Deserialize::deserialize(deserializer)?;

        if v.is_null() {
            return Ok(Params::None);
        }
        Err(D::Error::custom("Invalid Pub-Sub parameters"))

        // from_value(v.clone()).map(Params::Logs)
        //     .map_err(|e| D::Error::custom(format!("Invalid Pub-Sub parameters: {}", e)))
    }
}
