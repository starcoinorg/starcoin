// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::{ensure, Error, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
#[cfg(any(test, feature = "fuzzing"))]
use rand::{rngs::OsRng, RngCore};
use schemars::{self, JsonSchema};
use serde::{de, ser, Deserialize, Serialize};
use std::str::FromStr;
use std::{
    convert::{TryFrom, TryInto},
    fmt,
};
/// A struct that represents a globally unique id for an Event stream that a user can listen to.
/// By design, the lower part of EventKey is the same as account address.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EventKey(#[schemars(with = "String")] [u8; EventKey::LENGTH]);

impl EventKey {
    /// Construct a new EventKey from a byte array slice.
    pub fn new(key: [u8; Self::LENGTH]) -> Self {
        Self(key)
    }

    /// The number of bytes in an EventKey.
    pub const LENGTH: usize = AccountAddress::LENGTH + 8;

    /// Get the byte representation of the event key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert event key into a byte array.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Get the account address part in this event key
    pub fn get_creator_address(&self) -> AccountAddress {
        let mut arr = [0u8; AccountAddress::LENGTH];
        arr.copy_from_slice(&self.0[Self::LENGTH - AccountAddress::LENGTH..]);

        AccountAddress::new(arr)
    }

    /// If this is the `ith` EventKey` created by `get_creator_address()`, return `i`
    pub fn get_creation_number(&self) -> u64 {
        u64::from_le_bytes(self.0[0..8].try_into().unwrap())
    }

    #[cfg(any(test, feature = "fuzzing"))]
    /// Create a random event key for testing
    pub fn random() -> Self {
        let mut rng = OsRng;
        let salt = rng.next_u64();
        Self::new_from_address(&AccountAddress::random(), salt)
    }

    /// Create a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        let mut output = [0; Self::LENGTH];
        let (lhs, rhs) = output.split_at_mut(8);
        lhs.copy_from_slice(&salt.to_le_bytes());
        rhs.copy_from_slice(addr.as_ref());
        Self(output)
    }
}

impl From<EventKey> for [u8; EventKey::LENGTH] {
    fn from(event_key: EventKey) -> Self {
        event_key.0
    }
}

impl From<&EventKey> for [u8; EventKey::LENGTH] {
    fn from(event_key: &EventKey) -> Self {
        event_key.0
    }
}

impl ser::Serialize for EventKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            serializer.serialize_newtype_struct("EventKey", serde_bytes::Bytes::new(&self.0))
        }
    }
}

impl<'de> de::Deserialize<'de> for EventKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            Self::from_str(s.as_str()).map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            // See comment in serialize.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "EventKey")]
            struct Value<'a>(&'a [u8]);

            let value = Value::deserialize(deserializer)?;
            Self::try_from(value.0).map_err(<D::Error as ::serde::de::Error>::custom)
        }
    }
}

impl TryFrom<&[u8]> for EventKey {
    type Error = Error;

    /// Tries to convert the provided byte array into Event Key.
    fn try_from(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() == Self::LENGTH,
            "The Event Key {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; Self::LENGTH];
        addr.copy_from_slice(bytes);
        Ok(Self(addr))
    }
}

impl FromStr for EventKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let data = hex::decode(s)?;
        Self::try_from(data.as_slice())
    }
}

/// A Rust representation of an Event Handle Resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EventHandle {
    /// Number of events in the event stream.
    count: u64,
    /// The associated globally unique key that is used as the key to the EventStore.
    key: EventKey,
}

impl EventHandle {
    /// Constructs a new Event Handle
    pub fn new(key: EventKey, count: u64) -> Self {
        Self { key, count }
    }

    /// Return the key to where this event is stored in EventStore.
    pub fn key(&self) -> &EventKey {
        &self.key
    }
    /// Return the counter for the handle
    pub fn count(&self) -> u64 {
        self.count
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn count_mut(&mut self) -> &mut u64 {
        &mut self.count
    }

    #[cfg(any(test, feature = "fuzzing"))]
    /// Create a random event handle for testing
    pub fn random_handle(count: u64) -> Self {
        Self {
            key: EventKey::random(),
            count,
        }
    }

    /// Derive a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        Self {
            key: EventKey::new_from_address(addr, salt),
            count: 0,
        }
    }
}

impl fmt::LowerHex for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_bytes()))
    }
}

impl fmt::Display for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "0x{:#x}", self)
    }
}
