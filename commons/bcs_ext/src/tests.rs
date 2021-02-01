use super::*;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct Test {
    f: u64,
}

#[test]
fn test_serialize() {
    let t1 = Test { f: 1 };
    let bytes = t1.encode().unwrap();
    let t2 = Test::decode(bytes.as_slice()).unwrap();
    assert_eq!(t1, t2);
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
struct One {
    pub num_field: u64,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
struct Two {
    pub str_field: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
struct Three {
    pub bool_field: bool,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
enum TestEnumV1 {
    One(One),
    Two(Two),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
enum TestEnumV2 {
    One(One),
    Two(Two),
    Three(Three),
}

#[test]
fn test_bcs_enum_compact() {
    let one_v1 = TestEnumV1::One(One::default());
    let two_v1 = TestEnumV1::Two(Two::default());
    let one_v1_bytes = bcs::to_bytes(&one_v1).unwrap();
    let two_v1_bytes = bcs::to_bytes(&two_v1).unwrap();
    let one_v2 = bcs::from_bytes::<TestEnumV2>(one_v1_bytes.as_slice()).unwrap();
    let two_v2 = bcs::from_bytes::<TestEnumV2>(two_v1_bytes.as_slice()).unwrap();
    assert!(matches!(one_v2, TestEnumV2::One(_)));
    assert!(matches!(two_v2, TestEnumV2::Two(_)));
    let one_v2_bytes = bcs::to_bytes(&one_v2).unwrap();
    let one_v1_2 = bcs::from_bytes::<TestEnumV1>(one_v2_bytes.as_slice()).unwrap();
    assert_eq!(one_v1, one_v1_2);

    let three_v2 = TestEnumV2::Three(Three::default());
    let three_bytes = bcs::to_bytes(&three_v2).unwrap();
    let three_v1 = bcs::from_bytes::<TestEnumV1>(three_bytes.as_slice());
    assert!(three_v1.is_err());
}
