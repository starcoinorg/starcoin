use starcoin_transactional_test_harness::run_test;

datatest_stable::harness!(run_test, "tests", r".*\.(mvir|move)$");
