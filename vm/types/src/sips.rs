// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::core_code_address;
use crate::identifier::IdentStr;
use move_core_types::language_storage::ModuleId;
use once_cell::sync::Lazy;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug)]
pub struct SIP {
    pub id: u64,
    pub module_name: &'static str,
    pub url: &'static str,
}

impl SIP {
    pub fn module_id(&self) -> ModuleId {
        ModuleId::new(
            core_code_address(),
            IdentStr::new(self.module_name)
                .expect("SIP_n is a valid ident.")
                .into(),
        )
    }
}

pub static SIP_2: SIP = SIP {
    id: 2,
    module_name: "SIP_2",
    url: "https://github.com/starcoinorg/SIPs/tree/master/sip-1",
};

pub static SIP_3: SIP = SIP {
    id: 3,
    module_name: "SIP_3",
    url: "https://github.com/starcoinorg/SIPs/tree/master/sip-1",
};

pub static SIPS: Lazy<Vec<SIP>> = Lazy::new(|| vec![SIP_2, SIP_3]);
