// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::batch::WriteBatch;
use crate::define_storage;
use crate::storage::CodecStorage;
use crate::BRANCH_PREFIX_NAME;
use anyhow::Result;
use crypto::HashValue;
use std::sync::Arc;

define_storage!(BranchStorage, HashValue, HashValue, BRANCH_PREFIX_NAME);
