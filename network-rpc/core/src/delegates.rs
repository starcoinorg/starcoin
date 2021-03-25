// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::prelude::future::BoxFuture;
use futures::Future;
use starcoin_types::peer_info::PeerId;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub trait RpcMethod: Send + Sync + 'static {
    fn call(&self, peer_id: PeerId, params: Vec<u8>) -> BoxFuture<Result<Vec<u8>>>;
}

struct DelegateAsyncMethod<T, F> {
    delegate: Arc<T>,
    closure: F,
}

impl<T, F, I> RpcMethod for DelegateAsyncMethod<T, F>
where
    F: Fn(Arc<T>, PeerId, Vec<u8>) -> I,
    I: Future<Output = Result<Vec<u8>>> + Send + Unpin + 'static,
    T: Send + Sync + 'static,
    F: Send + Sync + 'static,
{
    fn call(&self, peer_id: PeerId, params: Vec<u8>) -> BoxFuture<Result<Vec<u8>>> {
        let closure = &self.closure;
        Box::pin(closure(self.delegate.clone(), peer_id, params))
    }
}

pub struct IoDelegate<T>
where
    T: Send + Sync + 'static,
{
    delegate: Arc<T>,
    methods: HashMap<Cow<'static, str>, Arc<dyn RpcMethod>>,
}

impl<T> IoDelegate<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(delegate: Arc<T>) -> Self {
        IoDelegate {
            delegate,
            methods: Default::default(),
        }
    }

    pub fn add_method<F, I>(&mut self, name: &'static str, method: F)
    where
        F: Fn(Arc<T>, PeerId, Vec<u8>) -> I,
        F: Send + Sync + 'static,
        I: Future<Output = Result<Vec<u8>>> + Send + Unpin + 'static,
    {
        self.methods.insert(
            Cow::from(name),
            Arc::new(DelegateAsyncMethod {
                delegate: self.delegate.clone(),
                closure: method,
            }),
        );
    }
}

impl<T> IntoIterator for IoDelegate<T>
where
    T: Send + Sync + 'static,
{
    type Item = (Cow<'static, str>, Arc<dyn RpcMethod>);
    type IntoIter = std::collections::hash_map::IntoIter<Cow<'static, str>, Arc<dyn RpcMethod>>;

    fn into_iter(self) -> Self::IntoIter {
        self.methods.into_iter()
    }
}
