// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub enum MultiDryRunTransaction {
    VM1(starcoin_vm_types::transaction::DryRunTransaction),
    VM2(starcoin_vm2_vm_types::transaction::DryRunTransaction),
}
