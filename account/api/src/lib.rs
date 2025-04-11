// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod error;
pub mod message;
mod provider;
mod service;
mod setting;
mod types;
pub use provider::*;
pub use service::*;
pub use setting::*;
pub use types::*;
pub type AccountResult<T> = Result<T, error::AccountError>;
