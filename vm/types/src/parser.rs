// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Type tag parsing for strings that may come from RPC callers.
//!
//! Move's parser recurses once per `<` with no depth limit, and a parsed tag deeper than BCS's
//! container limit (500) panics later when the state tree hashes it as a key. Both were
//! reachable with one public RPC call, so nesting is checked before parsing.

use anyhow::{ensure, Result};
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::parser as move_parser;

pub use move_core_types::parser::parse_transaction_argument;

/// Deepest `<…>` nesting accepted. On-chain types nest a few levels; this leaves ample room
/// while staying far below the parser's stack use and BCS's container depth.
pub const MAX_TYPE_TAG_NESTING: usize = 64;

fn check_nesting(s: &str) -> Result<()> {
    let mut depth = 0usize;
    for c in s.bytes() {
        match c {
            b'<' => {
                depth += 1;
                ensure!(
                    depth <= MAX_TYPE_TAG_NESTING,
                    "type tag nests deeper than {} levels",
                    MAX_TYPE_TAG_NESTING
                );
            }
            b'>' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

/// Nesting depth of a decoded type tag: `u8` is 0, `vector<u8>` is 1. Used on type arguments
/// that arrive already decoded (BCS transactions), where the string check cannot apply.
pub fn type_tag_nesting(t: &TypeTag) -> usize {
    let mut max = 0;
    let mut stack = vec![(t, 0usize)];
    while let Some((t, depth)) = stack.pop() {
        max = max.max(depth);
        match t {
            TypeTag::Vector(inner) => stack.push((inner, depth + 1)),
            TypeTag::Struct(st) => stack.extend(st.type_params.iter().map(|p| (p, depth + 1))),
            _ => {}
        }
    }
    max
}

pub fn parse_type_tag(s: &str) -> Result<TypeTag> {
    check_nesting(s)?;
    move_parser::parse_type_tag(s)
}

pub fn parse_type_tags(s: &str) -> Result<Vec<TypeTag>> {
    check_nesting(s)?;
    move_parser::parse_type_tags(s)
}

pub fn parse_struct_tag(s: &str) -> Result<StructTag> {
    check_nesting(s)?;
    move_parser::parse_struct_tag(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested(levels: usize) -> String {
        format!(
            "0x1::Account::Balance<{}u8{}>",
            "vector<".repeat(levels - 1),
            ">".repeat(levels - 1)
        )
    }

    #[test]
    fn accepts_real_types_and_the_limit() {
        parse_struct_tag("0x1::Account::Balance<0x1::STC::STC>").unwrap();
        parse_type_tag("vector<vector<u8>>").unwrap();
        parse_struct_tag(&nested(MAX_TYPE_TAG_NESTING)).unwrap();
    }

    #[test]
    fn decoded_nesting_matches_the_string_limit() {
        assert_eq!(type_tag_nesting(&TypeTag::U8), 0);
        let t = parse_type_tag(&nested(MAX_TYPE_TAG_NESTING)).unwrap();
        assert_eq!(type_tag_nesting(&t), MAX_TYPE_TAG_NESTING);
    }

    #[test]
    fn rejects_deeper_nesting_before_parsing() {
        for levels in [MAX_TYPE_TAG_NESTING + 1, 600, 200_000] {
            let err = parse_struct_tag(&nested(levels)).unwrap_err();
            assert!(err.to_string().contains("nests deeper"), "{}", err);
            assert!(parse_type_tag(&nested(levels)).is_err());
            assert!(parse_type_tags(&nested(levels)).is_err());
        }
    }
}
