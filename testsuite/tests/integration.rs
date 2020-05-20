use std::time::Duration;

use cucumber::{after, before, cucumber, Steps, StepsBuilder};
use starcoin_config::NodeConfig;
use starcoin_node::NodeHandle;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use std::sync::Arc;
use std::thread;

#[derive(Default)]
pub struct World {
    node_config: Option<Arc<NodeConfig>>,
    storage: Option<Arc<Storage>>,
    node_handle: Option<NodeHandle>,
}

impl cucumber::World for World {}

pub fn steps() -> Steps<World> {
    let mut builder: StepsBuilder<World> = Default::default();
    builder
        .given("a node config", |world: &mut World, _step| {
            let config = NodeConfig::random_for_test();
            world.node_config = Some(Arc::new(config))
        })
        .given("a storage", |world: &mut World, _step| {
            let cache_storage = Arc::new(CacheStorage::new());
            let db_storage = Arc::new(DBStorage::new(starcoin_config::temp_path().as_ref()));
            let storage = Storage::new(StorageInstance::new_cache_and_db_instance(
                cache_storage,
                db_storage,
            ))
            .unwrap();

            world.storage = Some(Arc::new(storage))
        })
        .given("a node handle", |world: &mut World, _step| {
            let handle = starcoin_node::run_dev_node(world.node_config.as_ref().unwrap().clone());
            world.node_handle = Some(handle)
        })
        .then("node handle stop", |world: &mut World, _step| {
            thread::sleep(Duration::from_secs(10));
            let handle = world.node_handle.as_ref().unwrap();
            handle.clone().stop().unwrap()
        });
    builder.build()
}

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {

});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {

});

// A setup function to be called before everything else
fn setup() {}

cucumber! {
    features: "./features", // Path to our feature files
    world: World, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        crate::steps // the `steps!` macro creates a `steps` function in a module
    ],
    setup: setup, // Optional; called once before everything
    before: &[
        a_before_fn // Optional; called before each scenario
    ],
    after: &[
        an_after_fn // Optional; called after each scenario
    ]
}
