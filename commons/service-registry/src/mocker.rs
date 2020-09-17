// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ActorService, ServiceContext};
use log::info;
use std::any::Any;

pub trait MockHandler<S>: Send
where
    S: ActorService,
{
    fn handle(&mut self, r: Box<dyn Any>, ctx: &mut ServiceContext<S>) -> Box<dyn Any>;

    fn handle_event(&mut self, msg: Box<dyn Any>, _ctx: &mut ServiceContext<S>) {
        info!(
            "Mock {} handler process event msg: {:?}",
            S::service_name(),
            msg
        );
    }
}

pub type MockFn<S> = Box<dyn FnMut(Box<dyn Any>, &mut ServiceContext<S>) -> Box<dyn Any> + Send>;

pub fn mock<S, F>(f: F) -> MockFn<S>
where
    S: ActorService,
    F: FnMut(Box<dyn Any>, &mut ServiceContext<S>) -> Box<dyn Any> + Send + 'static,
{
    Box::new(f)
}

impl<S> MockHandler<S> for MockFn<S>
where
    S: ActorService,
{
    fn handle(&mut self, r: Box<dyn Any>, ctx: &mut ServiceContext<S>) -> Box<dyn Any> {
        self(r, ctx)
    }
}
