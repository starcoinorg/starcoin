// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::hash::CryptoHash;

use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContractEvent {}
