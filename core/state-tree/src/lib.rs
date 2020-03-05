// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use std::cell::RefCell;
use std::sync::Arc;

pub mod mock;
mod state_tree;
#[cfg(test)]
mod state_tree_test;

pub use state_tree::{StateNode, StateNodeStore, StateProof, StateTree};
