// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{Address, Transaction, H256, U256};
use ethereum_types::BigEndianHash;

#[derive(Debug, Default, Clone)]
pub struct TransactionBuilder {
	nonce: U256,
	gas_price: U256,
	gas: U256,
	sender: Address,
	mem_usage: usize,
}

impl TransactionBuilder {
	pub fn tx(&self) -> Self {
		self.clone()
	}

	pub fn nonce(mut self, nonce: usize) -> Self {
		self.nonce = U256::from(nonce);
		self
	}

	pub fn gas_price(mut self, gas_price: usize) -> Self {
		self.gas_price = U256::from(gas_price);
		self
	}

	pub fn sender(mut self, sender: u64) -> Self {
		self.sender = Address::from_low_u64_be(sender);
		self
	}

	pub fn mem_usage(mut self, mem_usage: usize) -> Self {
		self.mem_usage = mem_usage;
		self
	}

	pub fn new(self) -> Transaction {
		let hash: U256 = self.nonce
			^ (U256::from(100) * self.gas_price)
			^ (U256::from(100_000) * U256::from(self.sender.to_low_u64_be()));
		Transaction {
			hash: H256::from_uint(&hash),
			nonce: self.nonce,
			gas_price: self.gas_price,
			gas: 21_000.into(),
			sender: self.sender,
			mem_usage: self.mem_usage,
		}
	}
}
