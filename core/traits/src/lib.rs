// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod mock;
mod txpool;

pub use txpool::TxPool;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traits() {}
}
