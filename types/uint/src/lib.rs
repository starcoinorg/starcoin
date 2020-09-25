// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::assign_op_pattern)]

use serde::{de, ser, Deserialize, Serialize, Serializer};
use starcoin_crypto::HashValue;
use std::convert::TryFrom;
use uint::*;
construct_uint! {
    pub struct U256(4);
}

construct_uint! {
    pub struct U512(8);
}
#[macro_export]
macro_rules! impl_uint_serde {
    ($name: ident, $len: expr) => {
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut bytes = [0u8; $len * 8];
                self.to_big_endian(&mut bytes);
                if serializer.is_human_readable() {
                    serializer.serialize_str(&hex::encode(&bytes))
                } else {
                    use ser::SerializeTuple;
                    let mut seq = serializer.serialize_tuple($len * 8)?;
                    for byte in &bytes[..] {
                        seq.serialize_element(byte)?;
                    }
                    seq.end()
                }
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct ByteArrayVisitor;

                impl<'de> de::Visitor<'de> for ByteArrayVisitor {
                    type Value = $name;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter.write_str("bytesArray of length $len*8")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<$name, A::Error>
                    where
                        A: de::SeqAccess<'de>,
                    {
                        use de::Error;
                        let mut arr = [0u8; $len * 8];
                        for (i, byte) in arr.iter_mut().enumerate() {
                            *byte = seq
                                .next_element()?
                                .ok_or_else(|| Error::invalid_length(i, &self))?;
                        }
                        Ok($name::from_big_endian(&arr))
                    }
                }
                struct HexVisitor;

                impl<'de> de::Visitor<'de> for HexVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "A hex string")
                    }

                    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                        let s = hex::decode(v).map_err(E::custom)?;
                        Ok($name::from_big_endian(&s))
                    }

                    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                        self.visit_str(&v)
                    }
                }
                if deserializer.is_human_readable() {
                    deserializer.deserialize_str(HexVisitor)
                } else {
                    deserializer
                        .deserialize_tuple($len * 8, ByteArrayVisitor)
                        .map(|bytes| bytes.into())
                }
            }
        }
    };
}

impl_uint_serde!(U256, 4);

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Overflow,
}

impl<'a> TryFrom<&'a U512> for U256 {
    type Error = Error;

    fn try_from(value: &'a U512) -> Result<U256, Error> {
        let U512(ref arr) = *value;
        if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
            return Err(Error::Overflow);
        }
        let mut ret = [0; 4];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        Ok(U256(ret))
    }
}

impl<'a> From<&'a U256> for U512 {
    fn from(value: &'a U256) -> U512 {
        let U256(ref arr) = *value;
        let mut ret = [0; 8];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        U512(ret)
    }
}

impl From<HashValue> for U256 {
    fn from(hash: HashValue) -> U256 {
        U256::from(hash.to_vec().as_slice())
    }
}

impl Into<HashValue> for U256 {
    fn into(self) -> HashValue {
        let mut bytes = [0u8; 32];
        self.to_big_endian(&mut bytes);
        HashValue::from_slice(&bytes).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_canonical_serialization::SCSCodec;
    use starcoin_crypto::HashValue;
    use starcoin_types::block::BlockHeader;

    #[test]
    fn test() {
        let u = U256::from(1024);
        let encoded = u.encode().unwrap();
        let u_decoded = U256::decode(&encoded).unwrap();
        assert_eq!(u, u_decoded);
        let max_hash: HashValue = U256::max_value().into();
        let max_u256: U256 = max_hash.into();
        assert_eq!(max_u256, U256::max_value());
        let h = BlockHeader::random();
        let h_encode = h.encode().unwrap();
        let h_decode = BlockHeader::decode(&h_encode).unwrap();
        assert_eq!(h, h_decode);
        let human_encode = serde_json::to_string_pretty(&U256::max_value()).unwrap();
        let human_decode: U256 = serde_json::from_str(&human_encode).unwrap();
        assert_eq!(human_decode, U256::max_value());
    }
}
