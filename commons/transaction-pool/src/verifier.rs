// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::VerifiedTransaction;

/// Transaction verification.
///
/// Verifier is responsible to decide if the transaction should even be considered for pool inclusion.
pub trait Verifier<U> {
	/// Verification error.
	type Error;

	/// Verified transaction.
	type VerifiedTransaction: VerifiedTransaction;

	/// Verifies a `UnverifiedTransaction` and produces `VerifiedTransaction` instance.
	fn verify_transaction(&self, tx: U) -> Result<Self::VerifiedTransaction, Self::Error>;
}
