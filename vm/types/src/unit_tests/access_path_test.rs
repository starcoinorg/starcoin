use crate::access_path::{AccessPath, DataType};
use crate::account_address::AccountAddress;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use std::str::FromStr;

#[test]
fn test_data_type() {
    let (_address, data_path) = AccessPath::random_resource().into_inner();
    assert_eq!(data_path.data_type(), DataType::RESOURCE);

    let (_address, data_path) = AccessPath::random_code().into_inner();
    assert_eq!(data_path.data_type(), DataType::CODE);
}

#[test]
fn test_access_path_str_valid() {
    let r1 = format!(
        "{}/1/0x00000000000000000000000000000001::Account::Account",
        AccountAddress::random()
    );
    let test_cases = vec!["0x00000000000000000000000000000000/0/Account",
                          "0x00000000000000000000000000000001/0/Account",
                          "0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Account::Account",
                          "0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Account::Balance<0x00000000000000000000000000000001::STC::STC>",
                          r1.as_str()];
    for case in test_cases {
        let access_path = AccessPath::from_str(case).unwrap();
        assert_eq!(case.to_owned(), access_path.to_string())
    }
}

#[test]
fn test_access_path_str_invalid() {
    //invalid address
    let r1 = format!(
        "{}00/1/0x00000000000000000000000000000001::Account::Account",
        AccountAddress::random()
    );
    let test_cases = vec![
        // invalid struct tag
        "0x00000000000000000000000000000001/1/Account",
        // invalid module name
        "0x00000000000000000000000000000001/0/0x00000000000000000000000000000001::Account::Account",
        //invalid data type
        "0x00000000000000000000000000000001/3/Account",
        //too many `/`
        "0x00000000000000000000000000000001/0/Account/xxx",
        "0x00000000000000000000000000000001/0//Account",
        //too less '`'
        "0x00000000000000000000000000000001/1",
        r1.as_str(),
    ];
    for case in test_cases {
        let access_path = AccessPath::from_str(case);
        assert!(
            access_path.is_err(),
            "expect err in access_path case: {}, but got ok",
            case
        );
    }
}

#[test]
fn test_bad_case_from_protest() {
    let access_path_str =
        "0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::a::A_";
    let access_path = AccessPath::from_str(access_path_str);
    assert!(access_path.is_ok());

    //The module name start with '_' will will encounter parse error
    //This may be the parser error, or the identity's arbitrary error
    let access_path_str =
        "0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::_a::A";
    let access_path = AccessPath::from_str(access_path_str);
    assert!(access_path.is_err());
}

proptest! {
        //TODO enable this test, when test_bad_case_from_protest is fixed.
        #[ignore]
        #[test]
        fn test_access_path(access_path in any::<AccessPath>()){
           let bytes = bcs_ext::to_bytes(&access_path).expect("access_path serialize should ok.");
           let access_path2 = bcs_ext::from_bytes::<AccessPath>(bytes.as_slice()).expect("access_path deserialize should ok.");
           prop_assert_eq!(&access_path, &access_path2);
           let access_path_str = access_path.to_string();
           let access_path3 = AccessPath::from_str(access_path_str.as_str()).expect("access_path from str should ok");
           prop_assert_eq!(&access_path, &access_path3);
           let json_str = serde_json::to_string(&access_path).expect("access_path to json str should ok");
           let access_path4 = serde_json::from_str::<AccessPath>(json_str.as_str()).expect("access_path from json str should ok");
            prop_assert_eq!(&access_path, &access_path4);
        }
}
