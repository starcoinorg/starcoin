// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssociationCapabilityResource {
    is_certified: bool,
}

impl AssociationCapabilityResource {
    pub fn is_certified(&self) -> bool {
        self.is_certified
    }
}
