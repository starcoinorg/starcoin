// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use dashmap::DashMap;
use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
use once_cell::sync::Lazy;
use rustc_hash::FxHasher;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::{Hash, Hasher},
};

type LayoutBucket = Vec<(MoveTypeLayout, bool)>;

// Bucket count soft limit for the global cache. If exceeded, clear to avoid unbounded growth.
const GLOBAL_CACHE_SOFT_LIMIT: usize = 100_000;

static GLOBAL_LAYOUT_IDENTIFIER_MAPPING_CACHE: Lazy<DashMap<u64, LayoutBucket>> =
    Lazy::new(DashMap::new);

#[inline]
fn hash_layout(layout: &MoveTypeLayout) -> u64 {
    let mut hasher = FxHasher::default();
    layout.hash(&mut hasher);
    hasher.finish()
}

#[inline]
fn lookup_bucket(bucket: &LayoutBucket, layout: &MoveTypeLayout) -> Option<bool> {
    bucket
        .iter()
        .find(|(cached_layout, _)| cached_layout == layout)
        .map(|(_, value)| *value)
}

#[inline]
fn insert_bucket(bucket: &mut LayoutBucket, layout: &MoveTypeLayout, value: bool) {
    if lookup_bucket(bucket, layout).is_none() {
        bucket.push((layout.clone(), value));
    }
}

#[inline]
fn maybe_trim_global_cache() {
    if GLOBAL_LAYOUT_IDENTIFIER_MAPPING_CACHE.len() > GLOBAL_CACHE_SOFT_LIMIT {
        GLOBAL_LAYOUT_IDENTIFIER_MAPPING_CACHE.clear();
    }
}

pub(crate) fn compute_layout_has_identifier_mappings(layout: &MoveTypeLayout) -> bool {
    match layout {
        MoveTypeLayout::Native(..) => true,
        MoveTypeLayout::Vector(inner) => compute_layout_has_identifier_mappings(inner),
        MoveTypeLayout::Struct(struct_layout) => match struct_layout {
            MoveStructLayout::Runtime(fields) => {
                fields.iter().any(compute_layout_has_identifier_mappings)
            }
            MoveStructLayout::WithFields(fields) => fields
                .iter()
                .any(|field| compute_layout_has_identifier_mappings(&field.layout)),
            MoveStructLayout::WithTypes { fields, .. } => fields
                .iter()
                .any(|field| compute_layout_has_identifier_mappings(&field.layout)),
        },
        _ => false,
    }
}

#[derive(Default)]
pub(crate) struct LayoutIdentifierMappingCache {
    // Fast-path for repeated checks on the same long-lived layout reference.
    // In vm-runtime callsites, layout references come from resolver/loader and are stable.
    last_layout_ptr: Cell<usize>,
    last_value: Cell<bool>,
    has_last: Cell<bool>,
    local_entries: RefCell<HashMap<u64, LayoutBucket>>,
}

impl LayoutIdentifierMappingCache {
    pub(crate) fn has_identifier_mappings(&self, layout: &MoveTypeLayout) -> bool {
        let ptr = layout as *const MoveTypeLayout as usize;
        if self.has_last.get() && self.last_layout_ptr.get() == ptr {
            return self.last_value.get();
        }

        let key = hash_layout(layout);

        if let Some(cached) = self
            .local_entries
            .borrow()
            .get(&key)
            .and_then(|bucket| lookup_bucket(bucket, layout))
        {
            self.last_layout_ptr.set(ptr);
            self.last_value.set(cached);
            self.has_last.set(true);
            return cached;
        }

        if let Some(cached) = GLOBAL_LAYOUT_IDENTIFIER_MAPPING_CACHE
            .get(&key)
            .and_then(|bucket| lookup_bucket(bucket.value(), layout))
        {
            self.local_entries
                .borrow_mut()
                .entry(key)
                .and_modify(|bucket| insert_bucket(bucket, layout, cached))
                .or_insert_with(|| vec![(layout.clone(), cached)]);
            self.last_layout_ptr.set(ptr);
            self.last_value.set(cached);
            self.has_last.set(true);
            return cached;
        }

        let computed = compute_layout_has_identifier_mappings(layout);
        self.local_entries
            .borrow_mut()
            .entry(key)
            .and_modify(|bucket| insert_bucket(bucket, layout, computed))
            .or_insert_with(|| vec![(layout.clone(), computed)]);
        GLOBAL_LAYOUT_IDENTIFIER_MAPPING_CACHE
            .entry(key)
            .and_modify(|bucket| insert_bucket(bucket, layout, computed))
            .or_insert_with(|| vec![(layout.clone(), computed)]);
        maybe_trim_global_cache();
        self.last_layout_ptr.set(ptr);
        self.last_value.set(computed);
        self.has_last.set(true);
        computed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use move_core_types::{
        identifier::Identifier,
        language_storage::StructTag,
        value::{IdentifierMappingKind, MoveFieldLayout},
    };

    fn id(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    fn runtime_native_layout() -> MoveTypeLayout {
        MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Native(
                IdentifierMappingKind::Aggregator,
                Box::new(MoveTypeLayout::U128),
            ),
        ]))
    }

    fn with_fields_native_layout() -> MoveTypeLayout {
        MoveTypeLayout::Struct(MoveStructLayout::WithFields(vec![
            MoveFieldLayout::new(id("a"), MoveTypeLayout::U64),
            MoveFieldLayout::new(
                id("b"),
                MoveTypeLayout::Native(
                    IdentifierMappingKind::Snapshot,
                    Box::new(MoveTypeLayout::U128),
                ),
            ),
        ]))
    }

    fn with_types_native_layout() -> MoveTypeLayout {
        MoveTypeLayout::Struct(MoveStructLayout::WithTypes {
            type_: StructTag {
                address: move_core_types::account_address::AccountAddress::ONE,
                module: id("M"),
                name: id("S"),
                type_args: vec![],
            },
            fields: vec![
                MoveFieldLayout::new(id("x"), MoveTypeLayout::U8),
                MoveFieldLayout::new(
                    id("y"),
                    MoveTypeLayout::Native(
                        IdentifierMappingKind::DerivedString,
                        Box::new(MoveTypeLayout::U64),
                    ),
                ),
            ],
        })
    }

    #[test]
    fn test_layout_identifier_mapping_cache_matches_compute() {
        let cache = LayoutIdentifierMappingCache::default();
        let layouts = vec![
            MoveTypeLayout::U64,
            runtime_native_layout(),
            with_fields_native_layout(),
            with_types_native_layout(),
            MoveTypeLayout::Vector(Box::new(MoveTypeLayout::Bool)),
        ];

        for layout in &layouts {
            let expected = compute_layout_has_identifier_mappings(layout);
            let cached_first = cache.has_identifier_mappings(layout);
            let cached_second = cache.has_identifier_mappings(layout);
            assert_eq!(expected, cached_first);
            assert_eq!(cached_first, cached_second);
        }
    }
}
