use move_vm_runtime::native_functions::NativeFunctionTable;

fn run_test_for_pkg(path_to_pkg: impl Into<String>) {}

pub fn starcoin_test_legacy_natives() -> NativeFunctionTable {}

#[test]
fn move_framework_legacy_unit_tests() {
    run_test_for_pkg("starcoin-framework-legacy");
}
