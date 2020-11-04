use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize_binary<S>(key: &[u8], s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if s.is_human_readable() {
        s.serialize_str(format!("0x{}", hex::encode(key)).as_str())
    } else {
        s.serialize_bytes(key)
    }
}

pub fn deserialize_binary<'de, D>(d: D) -> std::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    if d.is_human_readable() {
        let s = <String>::deserialize(d)?;
        let s = s.strip_prefix("0x").unwrap_or_else(|| &s);
        hex::decode(s).map_err(D::Error::custom)
    } else {
        serde_bytes::ByteBuf::deserialize(d).map(|b| b.into_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::{deserialize_binary, serialize_binary};
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(
            deserialize_with = "deserialize_binary",
            serialize_with = "serialize_binary"
        )]
        bytes: Vec<u8>,
    }

    #[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
    struct TestStructOrigin {
        bytes: Vec<u8>,
    }

    #[test]
    fn test_serialize_binary() {
        let data = TestStruct {
            bytes: vec![1, 2, 3],
        };

        {
            let se = serde_json::to_string(&data).unwrap();
            println!("{}", se);
            let de = serde_json::from_slice::<TestStruct>(se.as_bytes()).unwrap();
            assert_eq!(de, data);
        }

        {
            let se = scs::to_bytes(&data).unwrap();
            println!("{:?}", se);
            let de = scs::from_bytes::<TestStruct>(&se).unwrap();
            assert_eq!(de, data);

            let origin = TestStructOrigin {
                bytes: vec![1, 2, 3],
            };
            let origin_se = scs::to_bytes(&origin).unwrap();
            assert_eq!(se, origin_se);
        }
    }
}
