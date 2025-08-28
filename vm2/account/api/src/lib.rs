// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod error;
pub mod message;
mod provider;
mod rich_wallet;
mod service;
mod types;
pub use provider::*;
pub use rich_wallet::*;
pub use service::*;
pub use types::*;
pub type AccountResult<T> = std::result::Result<T, error::AccountError>;
