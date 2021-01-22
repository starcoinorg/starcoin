use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

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
        let s = s.strip_prefix("0x").unwrap_or(&s);
        hex::decode(s).map_err(D::Error::custom)
    } else {
        serde_bytes::ByteBuf::deserialize(d).map(|b| b.into_vec())
    }
}
pub fn serialize_to_string_opt<D, S>(data: &Option<D>, s: S) -> std::result::Result<S::Ok, S::Error>
where
    D: ToString + Serialize,
    S: Serializer,
{
    if s.is_human_readable() {
        match data {
            Some(d) => s.serialize_some(&d.to_string()),
            None => s.serialize_none(),
        }
    } else {
        data.serialize(s)
    }
}

pub fn deserialize_from_string_opt<'de, D, R>(d: D) -> std::result::Result<Option<R>, D::Error>
where
    D: Deserializer<'de>,
    R: FromStr + Deserialize<'de>,
    R::Err: Sized + std::error::Error,
{
    if d.is_human_readable() {
        let s = <Option<String>>::deserialize(d)?;
        s.map(|s| R::from_str(&s).map_err(D::Error::custom))
            .transpose()
    } else {
        Option::<R>::deserialize(d)
    }
}

pub fn serialize_to_string<D, S>(data: &D, s: S) -> std::result::Result<S::Ok, S::Error>
where
    D: ToString + Serialize,
    S: Serializer,
{
    if s.is_human_readable() {
        s.serialize_str(&data.to_string())
    } else {
        data.serialize(s)
    }
}

pub fn deserialize_from_string<'de, D, R>(d: D) -> std::result::Result<R, D::Error>
where
    D: Deserializer<'de>,
    R: FromStr + Deserialize<'de>,
    R::Err: Sized + std::error::Error,
{
    if d.is_human_readable() {
        let s = <String>::deserialize(d)?;
        R::from_str(&s).map_err(D::Error::custom)
    } else {
        R::deserialize(d)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        deserialize_binary, deserialize_from_string, serialize_binary, serialize_to_string,
    };
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
    #[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
    struct TestCustomU128Serde {
        #[serde(
            deserialize_with = "deserialize_from_string",
            serialize_with = "serialize_to_string"
        )]
        pub data: u128,
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
            let se = bcs_ext::to_bytes(&data).unwrap();
            println!("{:?}", se);
            let de = bcs_ext::from_bytes::<TestStruct>(&se).unwrap();
            assert_eq!(de, data);

            let origin = TestStructOrigin {
                bytes: vec![1, 2, 3],
            };
            let origin_se = bcs_ext::to_bytes(&origin).unwrap();
            assert_eq!(se, origin_se);
        }
    }

    #[test]
    fn test_serialize_u128() {
        let data = TestCustomU128Serde { data: 42 };

        {
            let se = serde_json::to_string(&data).unwrap();
            let expected = r#"{"data":"42"}"#;
            assert_eq!(&se, expected);
            let de = serde_json::from_str::<TestCustomU128Serde>(&se).unwrap();
            assert_eq!(de.data, data.data);
        }
    }
}
