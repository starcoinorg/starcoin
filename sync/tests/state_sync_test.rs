mod test_sync;

use config::SyncMode;

#[stest::test]
fn test_fast_sync() {
    test_sync::test_sync(SyncMode::FAST)
}
