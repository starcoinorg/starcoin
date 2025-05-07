// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub enum MultiDryRunOutputView {
    VM1(crate::types::DryRunOutputView),
    VM2(starcoin_vm2_types::view::dry_run_output_view::DryRunOutputView),
}
