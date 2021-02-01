use super::{deserialize_binary, deserialize_from_string, serialize_binary, serialize_to_string};
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
