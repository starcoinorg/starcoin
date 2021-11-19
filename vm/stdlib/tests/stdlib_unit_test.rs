use move_unit_test::UnitTestingConfig;
move_unit_test::register_move_unit_tests!(
    UnitTestingConfig::default_with_bound(Some(100_000_000)),
    "../modules",
    r".*\.move$"
);
