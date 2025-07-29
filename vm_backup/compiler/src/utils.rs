// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_command_line_common::files::{MOVE_COMPILED_EXTENSION, MOVE_EXTENSION};
use std::path::{Path, PathBuf};

/// Helper function to iterate through all the files in the given directory, skipping hidden files,
/// and return an iterator of their paths.
pub fn iterate_directory(path: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .map(::std::result::Result::unwrap)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .map_or(false, |s| !s.starts_with('.')) // Skip hidden files
        })
        .map(|entry| entry.path().to_path_buf())
}

pub fn filter_move_bytecode_files(
    dir_iter: impl Iterator<Item = PathBuf>,
) -> impl Iterator<Item = String> {
    filter_files(dir_iter, MOVE_COMPILED_EXTENSION.to_string())
        .map(|file| file.to_string_lossy().to_string())
}

pub fn filter_move_files(dir_iter: impl Iterator<Item = PathBuf>) -> impl Iterator<Item = String> {
    filter_files(dir_iter, MOVE_EXTENSION.to_string())
        .map(|file| file.to_string_lossy().to_string())
}

fn filter_files(
    dir_iter: impl Iterator<Item = PathBuf>,
    extension: String,
) -> impl Iterator<Item = PathBuf> {
    dir_iter.flat_map(move |path| {
        if path.extension()?.to_str()? == extension {
            Some(path)
        } else {
            None
        }
    })
}
