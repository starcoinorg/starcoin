use crate::token::stc::STCUnit;

#[test]
pub fn test_stc_unit_parse_basic() {
    let cases = vec![
        ("1 nanoSTC", 1u128),
        ("1 microSTC", 1000u128),
        ("1 milliSTC", 1000000u128),
        ("1 STC", 1000000000u128),
    ];
    for (s, v) in cases {
        assert_eq!(
            v,
            STCUnit::parse(s).unwrap().scaling(),
            "test case {} fail",
            s
        );
    }
}

#[test]
pub fn test_stc_unit_to_string_basic() {
    let cases = vec![
        (STCUnit::NanoSTC.value_of(1), "1 nanoSTC"),
        (STCUnit::MicroSTC.value_of(1), "1 microSTC"),
        (STCUnit::MilliSTC.value_of(1), "1 milliSTC"),
        (STCUnit::STC.value_of(1), "1 STC"),
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
pub fn test_to_string_and_parse() {
    let cases = vec![
        STCUnit::parse("1 STC").unwrap(),
        STCUnit::parse("1.0 STC").unwrap(),
        STCUnit::parse("1.1 STC").unwrap(),
        STCUnit::parse("1.01 STC").unwrap(),
        STCUnit::parse("1.11 STC").unwrap(),
    ];
    for case in cases {
        let s = case.to_string();
        let v2 = STCUnit::parse(s.as_str()).unwrap();
        assert_eq!(case.scaling(), v2.scaling(), "Case {:?} test fail", case);
        assert_eq!(v2.to_string(), s, "Case {:?} test fail.", case);
    }
}

#[test]
pub fn test_stc_unit_parse_decimal() {
    let cases = vec![
        ("1 nanoSTC", 1u128),
        ("1.1 microSTC", 1100u128),
        ("1.001 microSTC", 1001u128),
        ("1.000001 milliSTC", 1000001u128),
        ("1.000000001 STC", 1000000001u128),
    ];
    for (s, v) in cases {
        assert_eq!(v, STCUnit::parse(s).unwrap().scaling());
    }
}

#[test]
pub fn test_stc_unit_parse_decimal_ok() {
    let cases = vec![
        ("1.0 nanoSTC", true),
        ("1.1 nanoSTC", false),
        ("1.000 microSTC", true),
        ("1.0001 microSTC", false),
        ("1.000000 milliSTC", true),
        ("1.0000001 milliSTC", false),
        ("1.000000000 STC", true),
        ("1.0000000001 STC", false),
    ];
    for (s, v) in cases {
        assert_eq!(
            v,
            STCUnit::parse(s).is_ok(),
            "test case ({},{}) failed",
            s,
            v
        );
    }
}
