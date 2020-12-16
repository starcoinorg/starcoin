use anyhow::{bail, Result};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::num::NonZeroU32;
use std::str::FromStr;
#[derive(Debug, Clone, PartialEq)]
pub struct ApiQuotaConfig {
    pub max_burst: NonZeroU32,
    pub duration: QuotaDuration,
}

impl std::fmt::Display for ApiQuotaConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.max_burst, self.duration)
    }
}

impl FromStr for ApiQuotaConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, '/').collect();
        if parts.len() != 2 {
            bail!("invalid quota format");
        }
        let max_burst = parts[0].parse::<NonZeroU32>()?;
        let quota_duration = parts[1].parse::<QuotaDuration>()?;
        Ok(ApiQuotaConfig {
            max_burst,
            duration: quota_duration,
        })
    }
}

impl Serialize for ApiQuotaConfig {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
impl<'de> Deserialize<'de> for ApiQuotaConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        s.parse::<ApiQuotaConfig>().map_err(D::Error::custom)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QuotaDuration {
    Second,
    Minute,
    Hour,
}

impl std::fmt::Display for QuotaDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            QuotaDuration::Second => "s",
            QuotaDuration::Minute => "m",
            QuotaDuration::Hour => "h",
        };
        write!(f, "{}", s)
    }
}
impl FromStr for QuotaDuration {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let quota_duration = match s {
            "s" => QuotaDuration::Second,
            "m" => QuotaDuration::Minute,
            "h" => QuotaDuration::Hour,
            _ => bail!("invalid quota duration"),
        };
        Ok(quota_duration)
    }
}
