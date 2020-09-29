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

pub static STC_TOKEN_CODE: Lazy<TokenCode> = Lazy::new(|| {
    TokenCode::from_str(STC_TOKEN_CODE_STR).expect("Parse STC token code should success.")
});

static STC_IDENTIFIER: Lazy<Identifier> = Lazy::new(|| Identifier::new(STC_NAME).unwrap());

pub fn stc_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: STC_IDENTIFIER.clone(),
        name: STC_IDENTIFIER.clone(),
        type_params: vec![],
    })
}

pub const SYMBOL_NANOSTC: &str = "nanoSTC";
pub const SYMBOL_MICROSTC: &str = "microSTC";
pub const SYMBOL_MILLISTC: &str = "milliSTC";
pub const SYMBOL_STC: &str = STC_NAME;

pub static SCALE_NANOSTC: u32 = 0;
pub static SCALE_MICROSTC: u32 = 3;
pub static SCALE_MILLISTC: u32 = 6;
pub static SCALE_STC: u32 = 9;

#[derive(Clone, Copy, Debug)]
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
    pub fn value_of(self, value: u128) -> TokenValue<STCUnit> {
        TokenValue::new(value, self)
    }

    pub fn units() -> Vec<STCUnit> {
        vec![
            STCUnit::NanoSTC,
            STCUnit::MicroSTC,
            STCUnit::MilliSTC,
            STCUnit::STC,
        ]
    }

    pub fn parse(value: &str) -> Result<TokenValue<STCUnit>> {
        ensure!(
            !value.is_empty(),
            "Can not parse a empty string to TokenValue"
        );
        let parts: Vec<&str> = value.split(' ').collect();
        ensure!(parts.len() <= 2, "Too many blank in value: {}", value);
        let (decimal, unit) = if parts.len() == 1 {
            (parts[0], STCUnit::default())
        } else {
            (parts[0], parts[1].parse()?)
        };
        unit.parse(decimal)
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

    fn scale(&self) -> u32 {
        match self {
            Self::NanoSTC => SCALE_NANOSTC,
            Self::MicroSTC => SCALE_MICROSTC,
            Self::MilliSTC => SCALE_MILLISTC,
            Self::STC => SCALE_STC,
        }
    }
}

impl FromStr for STCUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        for unit in STCUnit::units() {
            if unit.symbol() == s {
                return Ok(unit);
            }
        }
        bail!("Unsupported unit type: {}", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::token::stc::STCUnit;

    #[test]
    pub fn test_stc_unit_parse_basic() {
        let cases = vec![
            ("1 nanoSTC", 1u128),
            ("1 microSTC", 1000u128),
            ("1 milliSTC", 1000000u128),
            ("1 STC", 1000000000u128),
        ];
        for (s, v) in cases {
            assert_eq!(
                v,
                STCUnit::parse(s).unwrap().scaling(),
                "test case {} fail",
                s
            );
        }
    }

    #[test]
    pub fn test_stc_unit_to_string_basic() {
        let cases = vec![
            (STCUnit::NanoSTC.value_of(1), "1 nanoSTC"),
            (STCUnit::MicroSTC.value_of(1), "1 microSTC"),
            (STCUnit::MilliSTC.value_of(1), "1 milliSTC"),
            (STCUnit::STC.value_of(1), "1 STC"),
        ];
        for (v, s) in cases {
            assert_eq!(
                v.to_string(),
                s.to_string(),
                "test case ({:?}, {}) fail.",
                v,
                s
            );
        }
    }

    #[test]
    pub fn test_to_string_and_parse() {
        let cases = vec![
            STCUnit::parse("1 STC").unwrap(),
            STCUnit::parse("1.0 STC").unwrap(),
            STCUnit::parse("1.1 STC").unwrap(),
            STCUnit::parse("1.01 STC").unwrap(),
            STCUnit::parse("1.11 STC").unwrap(),
        ];
        for case in cases {
            let s = case.to_string();
            let v2 = STCUnit::parse(s.as_str()).unwrap();
            assert_eq!(case.scaling(), v2.scaling(), "Case {:?} test fail", case);
            assert_eq!(v2.to_string(), s, "Case {:?} test fail.", case);
        }
    }

    #[test]
    pub fn test_stc_unit_parse_decimal() {
        let cases = vec![
            ("1 nanoSTC", 1u128),
            ("1.1 microSTC", 1100u128),
            ("1.001 microSTC", 1001u128),
            ("1.000001 milliSTC", 1000001u128),
            ("1.000000001 STC", 1000000001u128),
        ];
        for (s, v) in cases {
            assert_eq!(v, STCUnit::parse(s).unwrap().scaling());
        }
    }

    #[test]
    pub fn test_stc_unit_parse_decimal_ok() {
        let cases = vec![
            ("1.0 nanoSTC", true),
            ("1.1 nanoSTC", false),
            ("1.000 microSTC", true),
            ("1.0001 microSTC", false),
            ("1.000000 milliSTC", true),
            ("1.0000001 milliSTC", false),
            ("1.000000000 STC", true),
            ("1.0000000001 STC", false),
        ];
        for (s, v) in cases {
            assert_eq!(
                v,
                STCUnit::parse(s).is_ok(),
                "test case ({},{}) failed",
                s,
                v
            );
        }
    }
}
