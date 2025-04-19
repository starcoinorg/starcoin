use crate::types::StructTagView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_state_api::message::StateRequestVMType;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
pub enum VmType {
    #[schemars(with = "String")]
    MoveVm1,

    #[schemars(with = "String")]
    MoveVm2,
}

impl Into<StateRequestVMType> for VmType {
    fn into(self) -> StateRequestVMType {
        match self {
            Self::MoveVm1 => StateRequestVMType::MoveVm1,
            Self::MoveVm2 => StateRequestVMType::MoveVm2,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct GetResourceOption {
    pub decode: bool,
    pub state_root: Option<HashValue>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct GetCodeOption {
    pub resolve: bool,
    pub state_root: Option<HashValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct ListResourceOption {
    pub decode: bool,
    /// The state tree root, default is the latest block state root
    pub state_root: Option<HashValue>,
    pub start_index: usize,
    pub max_size: usize,
    pub resource_types: Option<Vec<StructTagView>>,
}

impl Default for ListResourceOption {
    fn default() -> Self {
        ListResourceOption {
            decode: false,
            state_root: None,
            start_index: 0,
            max_size: usize::MAX,
            resource_types: None,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(default)]
pub struct ListCodeOption {
    pub resolve: bool,
    /// The state tree root, default is the latest block state root
    pub state_root: Option<HashValue>,
    //TODO support filter by type and pagination
}
