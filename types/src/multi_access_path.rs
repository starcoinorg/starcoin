use anyhow::Result;
use proptest_derive::Arbitrary;
use schemars::{self, JsonSchema};
use starcoin_vm2_types::access_path::{AccessPath as AccessPath2, DataPath as DataPath2};
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::access_path::AccessPath;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, JsonSchema)]
#[schemars(with = "String")]
pub enum MultiAccessPath {
    VM1(AccessPath),
    VM2(AccessPath2),
}

impl MultiAccessPath {
    pub fn to_v1(self) -> Option<AccessPath> {
        match self {
            Self::VM1(access_path) => Some(access_path),
            Self::VM2(_) => None,
        }
    }

    pub fn to_v2(self) -> Option<AccessPath2> {
        match self {
            Self::VM1(_) => None,
            Self::VM2(access_path) => Some(access_path),
        }
    }

    pub fn to_state_key(self) -> Result<Option<StateKey>> {
        match self {
            Self::VM1(_) => Ok(None),
            Self::VM2(access_path) => {
                let state_key = match access_path.path {
                    DataPath2::Code(module_name) => {
                        StateKey::module(&access_path.address, &module_name)
                    }
                    DataPath2::Resource(struct_tag) => {
                        StateKey::resource(&access_path.address, &struct_tag)?
                    }
                    DataPath2::ResourceGroup(struct_tag) => {
                        StateKey::resource_group(&access_path.address, &struct_tag)
                    }
                };
                Ok(Some(state_key))
            }
        }
    }
}
