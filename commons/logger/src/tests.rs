use super::prelude::*;
use crate::LogLevelSpec;

#[test]
fn test_log() {
    let handle = super::init_for_test();
    debug!("debug message2.");
    info!("info message.");
    warn!("warn message.");
    error!("error message.");
    let handle2 = super::init_for_test();
    assert_eq!(handle.level(), handle2.level());
    assert_eq!(handle.log_path(), handle2.log_path());
    assert_eq!(handle.stderr(), handle2.stderr());
    let origin_level = handle.level();

    handle.update_level(LevelFilter::Off);

    assert_eq!(handle.level(), LevelFilter::Off);
    assert_eq!(handle.level(), handle2.level());

    handle.update_level(origin_level);
}

#[test]
fn test_log_level_spec() {
    let test_cases = vec![
        ("", LogLevelSpec::default()),
        (
            "info",
            LogLevelSpec {
                global_level: Some(LevelFilter::Info),
                module_levels: vec![],
            },
        ),
        (
            "debug,common=info,network=warn",
            LogLevelSpec {
                global_level: Some(LevelFilter::Debug),
                module_levels: vec![
                    ("common".to_string(), LevelFilter::Info),
                    ("network".to_string(), LevelFilter::Warn),
                ],
            },
        ),
    ];

    for (spec_str, expected) in test_cases {
        let actual = super::parse_spec(spec_str);
        assert_eq!(actual, expected);
    }
}
