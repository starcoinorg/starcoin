// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_vm_types::on_chain_config::Version;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum StdlibVersion {
    #[default]
    Latest,
    Version(VersionNumber),
}

type VersionNumber = u64;

impl StdlibVersion {
    pub fn new(version: u64) -> Self {
        if version == 0 {
            StdlibVersion::Latest
        } else {
            StdlibVersion::Version(version)
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            StdlibVersion::Latest => "latest".to_string(),
            StdlibVersion::Version(version) => format!("{}", version),
        }
    }

    pub fn version(&self) -> u64 {
        match self {
            StdlibVersion::Latest => 0,
            StdlibVersion::Version(version) => *version,
        }
    }

    pub fn is_latest(&self) -> bool {
        matches!(self, StdlibVersion::Latest)
    }

    pub fn compatible_with_previous(version: &StdlibVersion) -> bool {
        // currently only 4 is not compatible with previous version
        // Todo: need a better solution
        !matches!(version, StdlibVersion::Version(4))
    }
}

impl PartialOrd for StdlibVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StdlibVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (StdlibVersion::Latest, StdlibVersion::Latest) => Ordering::Equal,
            (StdlibVersion::Latest, _) => Ordering::Greater,
            (_, StdlibVersion::Latest) => Ordering::Less,
            (StdlibVersion::Version(self_v), StdlibVersion::Version(other_v)) => {
                self_v.cmp(other_v)
            }
        }
    }
}
impl FromStr for StdlibVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s {
            "latest" => Ok(StdlibVersion::Latest),
            s => Ok(Self::new(s.parse()?)),
        }
    }
}

impl Display for StdlibVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StdlibVersion::Latest => f.write_str("latest"),
            StdlibVersion::Version(version) => f.write_str(version.to_string().as_str()),
        }
    }
}

// migrate from Version::into_stdlib_version
impl From<Version> for StdlibVersion {
    fn from(v: Version) -> Self {
        if v.major == 0 {
            StdlibVersion::Latest
        } else {
            StdlibVersion::Version(v.major)
        }
    }
}

// convert starcoin_vm2_vm_types::on_chain_config::Version to StdlibVersion
impl From<starcoin_vm2_vm_types::on_chain_config::Version> for StdlibVersion {
    fn from(v: starcoin_vm2_vm_types::on_chain_config::Version) -> Self {
        if v.major == 0 {
            StdlibVersion::Latest
        } else {
            StdlibVersion::Version(v.major)
        }
    }
}
