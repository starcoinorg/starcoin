// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_rpc_api::types::FactoryAction;
use std::sync::atomic::{AtomicBool, Ordering};

static FACTORY_STATUS: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Serialize, Deserialize)]
pub struct TxFactoryStatusHandle {}

impl TxFactoryStatusHandle {
    pub fn handle_action(action: FactoryAction) -> bool {
        let _result = match action {
            FactoryAction::Stop => FACTORY_STATUS.compare_and_swap(true, false, Ordering::SeqCst),
            FactoryAction::Start => FACTORY_STATUS.compare_and_swap(false, true, Ordering::SeqCst),
            _ => true,
        };
        FACTORY_STATUS.load(Ordering::SeqCst)
    }
}
