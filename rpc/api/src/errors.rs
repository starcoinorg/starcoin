// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

pub fn invalid_params<T: fmt::Debug>(param: &str, details: T) -> anyhow::Error {
    anyhow::anyhow!("Couldn't parse parameters: {} ({:?})", param, details)
}
