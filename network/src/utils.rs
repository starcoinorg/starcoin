// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]

use std::time::Duration;
use futures::{FutureExt, Stream, StreamExt, stream::unfold};
use futures_timer::Delay;

pub fn interval(duration: Duration) -> impl Stream<Item=()> + Unpin {
	unfold((), move |_| {
		Delay::new(duration).map(|_| Some(((), ())))
	}).map(drop)
}
