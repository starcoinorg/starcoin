mod test_sync;

use config::SyncMode;

#[stest::test]
fn test_full_sync() {
    test_sync::test_sync(SyncMode::FULL)
}
