// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpc_core::{Error, ErrorCode, Value};
use std::fmt;

pub fn invalid_params<T: fmt::Debug>(param: &str, details: T) -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: format!("Couldn't parse parameters: {}", param),
        data: Some(Value::String(format!("{:?}", details))),
    }
}
