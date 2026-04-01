use anyhow::{bail, Result};
use move_vm2_core_types::move_resource::MoveStructType;
use schemars::{self, JsonSchema};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_vm2_types::access_path::{AccessPath as AccessPath2, DataPath as DataPath2};
use starcoin_vm2_types::account_config::AccountResource as AccountResource2;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::AccountResource;
use std::{fmt, str::FromStr};

use crate::multi_transaction::MultiAccountAddress;
use starcoin_vm_types::move_resource::MoveResource;

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, JsonSchema)]
#[schemars(with = "String")]
pub enum MultiAccessPath {
    VM1(AccessPath),
    VM2(AccessPath2),
}

impl Serialize for MultiAccessPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let s = match self {
                MultiAccessPath::VM1(ap) => format!("{}", ap),
                MultiAccessPath::VM2(ap2) => format!("{}", ap2),
            };
            serializer.serialize_str(&s)
        } else {
            match self {
                MultiAccessPath::VM1(ap) => {
                    serializer.serialize_newtype_variant("MultiAccessPath", 0, "VM1", ap)
                }
                MultiAccessPath::VM2(ap2) => {
                    serializer.serialize_newtype_variant("MultiAccessPath", 1, "VM2", ap2)
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for MultiAccessPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let parts: Vec<&str> = s.splitn(2, '/').collect();
            if parts.len() != 2 {
                return Err(D::Error::custom("Invalid format for MultiAccessPath"));
            }
            match parts[0] {
                "vm1" => {
                    let ap = AccessPath::from_str(parts[1]).map_err(D::Error::custom)?;
                    Ok(MultiAccessPath::VM1(ap))
                }
                "vm2" => {
                    let ap2 = AccessPath2::from_str(parts[1]).map_err(D::Error::custom)?;
                    Ok(MultiAccessPath::VM2(ap2))
                }
                _ => Err(D::Error::custom("Unknown VM type")),
            }
        } else {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum Inner {
                VM1(AccessPath),
                VM2(AccessPath2),
            }

            #[derive(Deserialize)]
            struct Wrapper {
                #[serde(flatten)]
                inner: Inner,
            }

            let Wrapper { inner } = Wrapper::deserialize(deserializer)?;
            match inner {
                Inner::VM1(ap) => Ok(MultiAccessPath::VM1(ap)),
                Inner::VM2(ap2) => Ok(MultiAccessPath::VM2(ap2)),
            }
        }
    }
}

impl FromStr for MultiAccessPath {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() != 2 {
            bail!("Invalid MultiAccessPath string format: {}", s);
        }

        match parts[0] {
            "vm1" => {
                let ap = AccessPath::from_str(parts[1])?;
                Ok(MultiAccessPath::VM1(ap))
            }
            "vm2" => {
                let ap2 = AccessPath2::from_str(parts[1])?;
                Ok(MultiAccessPath::VM2(ap2))
            }
            _ => bail!("Unknown VM type '{}'", parts[0]),
        }
    }
}

impl fmt::Debug for MultiAccessPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultiAccessPath::VM1(ap) => {
                write!(f, "MultiAccessPath::VM1({:?})", ap)
            }
            MultiAccessPath::VM2(ap2) => {
                write!(f, "MultiAccessPath::VM2({:?})", ap2)
            }
        }
    }
}

impl fmt::Display for MultiAccessPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MultiAccessPath::VM1(access_path) => {
                write!(f, "{}/{}", access_path.address, access_path.path)
            }
            MultiAccessPath::VM2(access_path) => {
                write!(f, "{}/{}", access_path.address, access_path.path)
            }
        }
    }
}

impl From<AccessPath> for MultiAccessPath {
    fn from(access_path: AccessPath) -> Self {
        MultiAccessPath::VM1(access_path)
    }
}

impl From<AccessPath2> for MultiAccessPath {
    fn from(access_path: AccessPath2) -> Self {
        MultiAccessPath::VM2(access_path)
    }
}

impl From<MultiAccountAddress> for MultiAccessPath {
    fn from(addr: MultiAccountAddress) -> Self {
        match addr {
            MultiAccountAddress::VM1(addr) => MultiAccessPath::VM1(
                AccessPath::resource_access_path(addr, AccountResource::struct_tag()),
            ),
            MultiAccountAddress::VM2(addr) => MultiAccessPath::VM2(
                AccessPath2::resource_access_path(addr, AccountResource2::struct_tag()),
            ),
        }
    }
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
