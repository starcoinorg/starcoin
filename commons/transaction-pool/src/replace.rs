// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! When queue limits are reached, decide whether to replace an existing transaction from the pool

use crate::{pool::Transaction, scoring::Choice};

/// Encapsulates a transaction to be compared, along with pooled transactions from the same sender
pub struct ReplaceTransaction<'a, T> {
	/// The transaction to be compared for replacement
	pub transaction: &'a Transaction<T>,
	/// Other transactions currently in the pool for the same sender
	pub pooled_by_sender: Option<&'a [Transaction<T>]>,
}

impl<'a, T> ReplaceTransaction<'a, T> {
	/// Creates a new `ReplaceTransaction`
	pub fn new(transaction: &'a Transaction<T>, pooled_by_sender: Option<&'a [Transaction<T>]>) -> Self {
		ReplaceTransaction { transaction, pooled_by_sender }
	}
}

impl<'a, T> ::std::ops::Deref for ReplaceTransaction<'a, T> {
	type Target = Transaction<T>;
	fn deref(&self) -> &Self::Target {
		&self.transaction
	}
}

/// Chooses whether a new transaction should replace an existing transaction if the pool is full.
pub trait ShouldReplace<T> {
	/// Decides if `new` should push out `old` transaction from the pool.
	///
	/// NOTE returning `InsertNew` here can lead to some transactions being accepted above pool limits.
	fn should_replace(&self, old: &ReplaceTransaction<'_, T>, new: &ReplaceTransaction<'_, T>) -> Choice;
}
