// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Transaction Pool options.
#[derive(Clone, Debug, PartialEq)]
pub struct Options {
	/// Maximal number of transactions in the pool.
	pub max_count: usize,
	/// Maximal number of transactions from single sender.
	pub max_per_sender: usize,
	/// Maximal memory usage.
	pub max_mem_usage: usize,
}

impl Default for Options {
	fn default() -> Self {
		Options { max_count: 1024, max_per_sender: 16, max_mem_usage: 8 * 1024 * 1024 }
	}
}
