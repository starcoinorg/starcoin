mod test_sync;

use config::SyncMode;

#[stest::test(timeout = 30)]
fn test_full_sync() {
    test_sync::test_sync(SyncMode::FULL)
}
