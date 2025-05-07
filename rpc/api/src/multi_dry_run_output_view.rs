// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::DryRunOutputView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm2_types::view::dry_run_output_view::DryRunOutputView as DryRunOutputViewV2;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum MultiDryRunOutputView {
    VM1(DryRunOutputView),
    VM2(DryRunOutputViewV2),
}

impl TryInto<DryRunOutputView> for MultiDryRunOutputView {
    type Error = ();

    fn try_into(self) -> Result<DryRunOutputView, Self::Error> {
        Ok(match self {
            MultiDryRunOutputView::VM1(view) => view,
            MultiDryRunOutputView::VM2(_view) => { unimplemented!() }
        })
    }
}
