// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{Actor, Context};
use std::time::Duration;

#[stest::test]
async fn my_async_test() {}

#[stest::test]
async fn my_async_test_with_result() -> anyhow::Result<()> {
    Ok(())
}

#[stest::test]
fn my_sync_test() {}

#[stest::test]
#[should_panic]
fn my_sync_test_false() {
    panic!("expect")
}

#[stest::test(timeout = 1)]
#[should_panic]
fn test_timeout() {
    std::thread::sleep(Duration::from_secs(6));
}

#[stest::test(timeout = 1)]
fn test_ok() -> anyhow::Result<()> {
    Ok(())
}

#[stest::test(timeout = 1)]
#[should_panic]
async fn test_async_timeout() {
    stest::actix_export::time::delay_for(Duration::from_secs(6)).await;
}

struct MyActor;
impl Actor for MyActor {
    type Context = Context<Self>;
}

#[stest::test(timeout = 10)]
async fn my_async_test_actor() {
    let myactor = MyActor {};
    myactor.start();
}
