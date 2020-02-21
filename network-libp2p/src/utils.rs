// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures03::{stream::unfold, FutureExt, Stream, StreamExt};
use futures_timer::Delay;
use std::time::Duration;

pub fn interval(duration: Duration) -> impl Stream<Item = ()> + Unpin {
    unfold((), move |_| Delay::new(duration).map(|_| Some(((), ())))).map(drop)
}
