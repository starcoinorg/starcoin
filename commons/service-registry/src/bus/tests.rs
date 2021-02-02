use super::*;
use crate::{RegistryAsyncService, RegistryService};
use actix::Arbiter;
use futures::StreamExt;
use log::debug;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone)]
struct MyMessage {}

#[stest::test]
async fn test_onshot() {
    let registry = RegistryService::launch();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    let bus2 = bus.clone();
    let arbiter = Arbiter::new();
    arbiter.exec_fn(move || loop {
        let result = bus2.broadcast(MyMessage {}).is_ok();
        debug!("broadcast result: {}", result);
        sleep(Duration::from_millis(50));
    });
    let msg = bus.oneshot::<MyMessage>().await.unwrap().await;
    assert!(msg.is_ok());
    let msg = bus.oneshot::<MyMessage>().await.unwrap().await;
    assert!(msg.is_ok());
}

#[stest::test]
async fn test_channel() {
    let registry = RegistryService::launch();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    let bus2 = bus.clone();
    let arbiter = Arbiter::new();
    arbiter.exec_fn(move || loop {
        let result = bus2.broadcast(MyMessage {}).is_ok();
        debug!("broadcast result: {}", result);
        sleep(Duration::from_millis(50));
    });
    let result = bus.channel::<MyMessage>().await;
    assert!(result.is_ok());
    let receiver = result.unwrap();
    let msgs: Vec<MyMessage> = receiver.take(3).collect().await;
    assert_eq!(3, msgs.len());

    let receiver2 = bus.channel::<MyMessage>().await.unwrap();
    let msgs: Vec<MyMessage> = receiver2.take(3).collect().await;
    assert_eq!(3, msgs.len());
}
