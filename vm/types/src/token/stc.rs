// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::token::token_code::TokenCode;
use crate::token::token_value::{TokenUnit, TokenValue};
use crate::{
    account_config::constants::CORE_CODE_ADDRESS,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use anyhow::{bail, ensure, Result};
use once_cell::sync::Lazy;
use std::str::FromStr;

pub const STC_NAME: &str = "STC";
pub const STC_TOKEN_CODE_STR: &str = "0x1::STC::STC";

pub static G_STC_TOKEN_CODE: Lazy<TokenCode> = Lazy::new(|| {
    TokenCode::from_str(STC_TOKEN_CODE_STR).expect("Parse STC token code should success.")
});

static G_STC_IDENTIFIER: Lazy<Identifier> = Lazy::new(|| Identifier::new(STC_NAME).unwrap());

pub fn stc_type_tag() -> TypeTag {
    TypeTag::Struct(Box::new(StructTag {
        address: CORE_CODE_ADDRESS,
        module: G_STC_IDENTIFIER.clone(),
        name: G_STC_IDENTIFIER.clone(),
        type_params: vec![],
    }))
}

pub const SYMBOL_NANOSTC: &str = "nanoSTC";
pub const SYMBOL_MICROSTC: &str = "microSTC";
pub const SYMBOL_MILLISTC: &str = "milliSTC";
pub const SYMBOL_STC: &str = STC_NAME;

pub const SYMBOL_NANOSTC_LOWER: &str = "nanostc";
pub const SYMBOL_MICROSTC_LOWER: &str = "microstc";
pub const SYMBOL_MILLISTC_LOWER: &str = "millistc";
pub const SYMBOL_STC_LOWER: &str = "stc";

pub static G_SCALE_NANOSTC: u32 = 0;
pub static G_SCALE_MICROSTC: u32 = 3;
pub static G_SCALE_MILLISTC: u32 = 6;
pub static G_SCALE_STC: u32 = 9;

#[derive(Clone, Copy, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum STCUnit {
    NanoSTC,
    MicroSTC,
    MilliSTC,
    STC,
}

impl Default for STCUnit {
    fn default() -> Self {
        Self::STC
    }
}

impl STCUnit {
    pub fn parse_value(value: &str) -> Result<TokenValue<Self>> {
        TokenValue::<Self>::from_str(value)
    }

    pub fn value_of(self, value: u128) -> TokenValue<Self> {
        TokenValue::new(value, self)
    }

    pub fn units() -> Vec<Self> {
        vec![Self::NanoSTC, Self::MicroSTC, Self::MilliSTC, Self::STC]
    }

    fn strip_unit_suffix(value: &str) -> (String, Self) {
        let value_lower = value.trim().to_lowercase();
        for unit in Self::units() {
            if let Some(v) = value_lower.strip_suffix(unit.symbol_lowercase()) {
                return (v.trim().to_string(), unit);
            }
        }
        (value_lower, Self::default())
    }
}

impl TokenUnit for STCUnit {
    fn symbol(&self) -> &'static str {
        match self {
            Self::NanoSTC => SYMBOL_NANOSTC,
            Self::MicroSTC => SYMBOL_MICROSTC,
            Self::MilliSTC => SYMBOL_MILLISTC,
            Self::STC => SYMBOL_STC,
        }
    }

    fn symbol_lowercase(&self) -> &'static str {
        match self {
            Self::NanoSTC => SYMBOL_NANOSTC_LOWER,
            Self::MicroSTC => SYMBOL_MICROSTC_LOWER,
            Self::MilliSTC => SYMBOL_MILLISTC_LOWER,
            Self::STC => SYMBOL_STC_LOWER,
        }
    }

    fn scale(&self) -> u32 {
        match self {
            Self::NanoSTC => G_SCALE_NANOSTC,
            Self::MicroSTC => G_SCALE_MICROSTC,
            Self::MilliSTC => G_SCALE_MILLISTC,
            Self::STC => G_SCALE_STC,
        }
    }
}

impl FromStr for STCUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        for unit in Self::units() {
            if unit.symbol().eq_ignore_ascii_case(s) {
                return Ok(unit);
            }
        }
        bail!("Unsupported unit type: {}", s)
    }
}

impl FromStr for TokenValue<STCUnit> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(!s.is_empty(), "Can not parse a empty string to TokenValue");
        let (decimal, unit) = STCUnit::strip_unit_suffix(s);
        unit.parse(decimal.as_str())
    }
}
