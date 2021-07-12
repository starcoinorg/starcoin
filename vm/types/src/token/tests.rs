use crate::token::stc::STCUnit;

#[test]
pub fn test_stc_unit_parse_basic() {
    let cases = vec![
        ("1nanoSTC", 1u128),
        ("1nanostc", 1u128),
        ("1 nanoSTC", 1u128),
        ("1microSTC", 1000u128),
        ("1microstc", 1000u128),
        ("1 microSTC", 1000u128),
        ("1milliSTC", 1000000u128),
        ("1millistc", 1000000u128),
        ("1 milliSTC", 1000000u128),
        ("1STC", 1000000000u128),
        ("1stc", 1000000000u128),
        ("1 STC", 1000000000u128),
    ];
    for (s, v) in cases {
        assert_eq!(
            v,
            STCUnit::parse_value(s).unwrap().scaling(),
            "test case {} fail",
            s
        );
    }
}

#[test]
pub fn test_stc_unit_to_string_basic() {
    let cases = vec![
        (STCUnit::NanoSTC.value_of(1), "1nanoSTC"),
        (STCUnit::MicroSTC.value_of(1), "1microSTC"),
        (STCUnit::MilliSTC.value_of(1), "1milliSTC"),
        (STCUnit::STC.value_of(1), "1STC"),
    ];
    for (v, s) in cases {
        assert_eq!(
            v.to_string(),
            s.to_string(),
            "test case ({:?}, {}) fail.",
            v,
            s
        );
    }
}

#[test]
pub fn test_to_string_and_parse_value() {
    let cases = vec![
        STCUnit::parse_value("1STC").unwrap(),
        STCUnit::parse_value("1.0STC").unwrap(),
        STCUnit::parse_value("1.1STC").unwrap(),
        STCUnit::parse_value("1.01STC").unwrap(),
        STCUnit::parse_value("1.11STC").unwrap(),
    ];
    for case in cases {
        let s = case.to_string();
        let v2 = STCUnit::parse_value(s.as_str()).unwrap();
        assert_eq!(case.scaling(), v2.scaling(), "Case {:?} test fail", case);
        assert_eq!(v2.to_string(), s, "Case {:?} test fail.", case);
    }
}

#[test]
pub fn test_stc_unit_parse_decimal() {
    let cases = vec![
        ("1nanoSTC", 1u128),
        ("1.1microSTC", 1100u128),
        ("1.001microSTC", 1001u128),
        ("1.000001milliSTC", 1000001u128),
        ("1.000000001STC", 1000000001u128),
    ];
    for (s, v) in cases {
        assert_eq!(v, STCUnit::parse_value(s).unwrap().scaling());
    }
}

#[test]
pub fn test_stc_unit_parse_decimal_ok() {
    let cases = vec![
        ("1.0nanoSTC", true),
        ("1.1nanoSTC", false),
        ("1.000microSTC", true),
        ("1.0001microSTC", false),
        ("1.000000milliSTC", true),
        ("1.0000001milliSTC", false),
        ("1.000000000STC", true),
        ("1.0000000001STC", false),
    ];
    for (s, v) in cases {
        assert_eq!(
            v,
            STCUnit::parse_value(s).is_ok(),
            "test case ({},{}) failed",
            s,
            v
        );
    }
}

#[test]
pub fn test_stc_uint_scaling() {
    assert_eq!(1000000000u128, STCUnit::STC.value_of(1).scaling());
    assert_eq!(
        111111111000000000u128,
        STCUnit::STC.value_of(111111111).scaling()
    );
    assert_eq!(
        1111111111000000000,
        STCUnit::STC.value_of(1111111111).scaling()
    );
}
