// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod chain;
mod chain_service;
mod chain_state;
pub mod mock;
mod txpool_service;

pub use chain::{Chain, ChainReader, ChainWriter};
pub use chain_service::{ChainAsyncService, ChainService};
pub use chain_state::{ChainState, ChainStateReader, ChainStateWriter};
pub use txpool_service::TxPoolAsyncService;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
