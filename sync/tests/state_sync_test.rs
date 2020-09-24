mod test_sync;

use config::SyncMode;

//FIXME
#[ignore]
#[stest::test]
fn test_fast_sync() {
    test_sync::test_sync(SyncMode::FAST)
}
