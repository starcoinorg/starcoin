// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{Address, HashValue, Transaction};

#[derive(Debug, Default, Clone)]
pub struct TransactionBuilder {
    nonce: u64,
    gas_price: u64,
    gas: u64,
    sender: Address,
    mem_usage: usize,
}

impl TransactionBuilder {
    pub fn tx(&self) -> Self {
        self.clone()
    }

    pub fn nonce(mut self, nonce: usize) -> Self {
        self.nonce = nonce as u64;
        self
    }

    pub fn gas_price(mut self, gas_price: usize) -> Self {
        self.gas_price = gas_price as u64;
        self
    }

    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = sender;
        self
    }

    pub fn mem_usage(mut self, mem_usage: usize) -> Self {
        self.mem_usage = mem_usage;
        self
    }

    pub fn new(self) -> Transaction {
        let hash = HashValue::random();
        Transaction {
            hash,
            nonce: self.nonce,
            gas_price: self.gas_price,
            gas: 21_000,
            sender: self.sender,
            mem_usage: self.mem_usage,
        }
    }
}
