// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub(crate) fn map_err(err: anyhow::Error) -> anyhow::Error {
    err
}

pub mod account_rpc;
pub mod contract_rpc;
mod helpers;
pub mod state_rpc;
