use serde::{Deserialize, Serialize};
use starcoin_vm_types::language_storage::TypeTag;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableHandleKey(pub u128);

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableInfoValue {
    pub key_type: TypeTag,
    pub value_type: TypeTag,
}

impl TableInfoValue {
    pub fn new(key_type: TypeTag, value_type: TypeTag) -> Self {
        Self {
            key_type,
            value_type,
        }
    }
}
