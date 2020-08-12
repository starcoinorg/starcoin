mod test_sync;

use config::SyncMode;

#[stest::test]
async fn test_fast_sync() {
    test_sync::test_sync(SyncMode::FAST).await
}
