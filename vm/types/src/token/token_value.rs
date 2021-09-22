// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};

pub trait TokenUnit: Clone + Copy {
    fn symbol(&self) -> &'static str;

    fn symbol_lowercase(&self) -> &'static str;

    fn scale(&self) -> u32;

    fn scaling_factor(&self) -> u128 {
        10u32.pow(self.scale()) as u128
    }

    fn scaling(&self, value: u128) -> u128 {
        self.scaling_factor() * value
    }

    fn parse(&self, input: &str) -> Result<TokenValue<Self>> {
        ensure!(!input.is_empty(), "Empty input not allowed for token unit");
        let symbol = self.symbol();
        let input = if let Some(stripped) = input.strip_suffix(symbol) {
            stripped
        } else {
            input
        };
        let input = input.trim();
        let parts: Vec<&str> = input.split('.').collect();
        ensure!(parts.len() <= 2, "Invalid decimal value, too many '.'");
        let h: u128 = parts[0].parse()?;
        let l: u128 = if parts.len() == 2 {
            let s = parts[1];
            let s = s.trim_end_matches('0');
            if s.is_empty() {
                0u128
            } else {
                let scale = self.scale();
                ensure!(
                    s.len() <= (scale as usize),
                    "Decimal part {} is overflow.",
                    s
                );
                let s = padding_zero(s, scale, false);
                s.parse()?
            }
        } else {
            0
        };
        TokenValue::new_with_parts(h, l, *self)
    }

    fn max(&self) -> u128 {
        u128::max_value() / self.scaling_factor()
    }
}

fn padding_zero(origin: &str, scale: u32, left: bool) -> String {
    let mut result = origin.to_string();
    let pad = "0".repeat((scale as usize) - origin.len());
    if left {
        result.insert_str(0, pad.as_str());
    } else {
        result.push_str(pad.as_str());
    }
    result
}

#[derive(Clone, Copy, Debug)]
pub struct TokenValue<U>
where
    U: TokenUnit,
{
    //value before decimal point
    h: u128,
    //value after decimal point
    l: u128,
    unit: U,
}

impl<U> std::fmt::Display for TokenValue<U>
where
    U: TokenUnit,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (p1, p2) = self.decimal();
        if p2 == 0 {
            write!(f, "{}{}", p1, self.unit.symbol())
        } else {
            let p2_str = padding_zero(p2.to_string().as_str(), self.unit.scale(), true);
            let p2_str = p2_str.trim_end_matches('0');
            write!(f, "{}.{}{}", p1, p2_str, self.unit.symbol())
        }
    }
}

impl<U> TokenValue<U>
where
    U: TokenUnit,
{
    pub fn new(value: u128, unit: U) -> Self {
        Self {
            h: value,
            l: 0,
            unit,
        }
    }

    pub fn new_with_parts(h: u128, l: u128, unit: U) -> Result<Self> {
        ensure!(
            h < unit.max(),
            "{} is too big than unit max value: {}",
            h,
            unit.max()
        );
        ensure!(
            l < unit.scaling_factor(),
            "Digits after the decimal point: {} contains digits more than scaling_factor: {}.",
            l,
            unit.scale()
        );
        Ok(Self { h, l, unit })
    }

    pub fn decimal(&self) -> (u128, u128) {
        (self.h, self.l)
    }

    pub fn scaling(&self) -> u128 {
        // h * scaling_factor + l
        self.h
            .checked_mul(self.unit.scaling_factor())
            .and_then(|v| v.checked_add(self.l))
            .expect("Scaling overflow.")
    }

    pub fn convert(self, unit: U) -> Self {
        if self.unit.scale() == unit.scale() {
            self
        } else {
            TokenValue::new(self.scaling(), unit)
        }
    }
}
