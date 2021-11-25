use move_unit_test::UnitTestingConfig;
use starcoin_natives::starcoin_natives;
move_unit_test::register_move_unit_tests!(
    {
        let mut c = UnitTestingConfig::default_with_bound(Some(100_000_000));
        c.verbose = true;
        c
    },
    "./modules",
    r".*\.move$",
    "./src",
    Some(starcoin_natives())
);
