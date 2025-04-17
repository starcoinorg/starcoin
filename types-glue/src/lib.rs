// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// #[macro_export]
// macro_rules! define_type_convert {
//     ($source_wrap_name:ident, $source_type:ty, $target_wrap_name:ident, $target_type:ty) => {
//         #[derive(Clone, Copy)]
//         pub struct $source_wrap_name(pub $source_type);
//         pub struct $target_wrap_name(pub $target_type);
//
//         impl std::ops::Deref for $source_wrap_name {
//             type Target = $source_type;
//
//             #[inline]
//             fn deref(&self) -> &Self::Target {
//                 &self.0
//             }
//         }
//
//         impl std::ops::Deref for $target_wrap_name {
//             type Target = $target_type;
//
//             #[inline]
//             fn deref(&self) -> &Self::Target {
//                 &self.0
//             }
//         }
//
//         impl From<$target_wrap_name> for $source_wrap_name {
//             fn from(src: $target_wrap_name) -> Self {
//                 Self(<$target_type>::new(src.0.into_bytes()))
//             }
//         }
//
//         impl From<$source_wrap_name> for $target_wrap_name {
//             fn from(src: $source_wrap_name) -> Self {
//                 Self(<$source_type>::new(src.0.into_bytes()))
//             }
//         }
//     };
// }

pub mod accounts;
mod account_state_set;
