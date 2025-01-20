// Copyright © Starcoin Foundation
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use starcoin_logger::prelude::*;
use tempfile::TempDir;
pub const MOVE_STDLIB_SOURCES_DIR: Dir = include_dir!("sources");
#[derive(Debug)]
pub struct SourceFiles {
    pub temp_dir: TempDir,
    pub files: Vec<String>,
}
pub static MOVE_STDLIB_SOURCE_FILES: Lazy<SourceFiles> = Lazy::new(|| {
    restore_sources(MOVE_STDLIB_SOURCES_DIR, "move-stdlib").expect("Restore source file error")
});
pub const STARCOIN_STDLIB_SOURCES_DIR: Dir = include_dir!("../starcoin-stdlib/sources");
pub static STARCOIN_STDLIB_SOURCE_FILES: Lazy<SourceFiles> = Lazy::new(|| {
    restore_sources(STARCOIN_STDLIB_SOURCES_DIR, "starcoin-stdlib")
        .expect("Restore source file error")
});
pub const STARCOIN_FRAMEWORK_SOURCES_DIR: Dir = include_dir!("../starcoin-framework/sources");
pub static STARCOIN_FRAMEWORK_SOURCE_FILES: Lazy<SourceFiles> = Lazy::new(|| {
    restore_sources(STARCOIN_FRAMEWORK_SOURCES_DIR, "starcoin-framework")
        .expect("Restore source file error")
});
pub fn move_stdlib_files() -> Vec<String> {
    MOVE_STDLIB_SOURCE_FILES.files.clone()
}
pub fn starcoin_stdlib_files() -> Vec<String> {
    STARCOIN_STDLIB_SOURCE_FILES.files.clone()
}
pub fn starcoin_framework_files() -> Vec<String> {
    STARCOIN_FRAMEWORK_SOURCE_FILES.files.clone()
}
//restore the sources files to a tempdir
fn restore_sources(dir: Dir, path: &str) -> anyhow::Result<SourceFiles> {
    let temp_dir = tempfile::tempdir()?;
    let sources_dir = temp_dir.path().join(path).join("sources");
    info!("restore {} sources in: {:?}", path, sources_dir);
    std::fs::create_dir_all(sources_dir.as_path())?;
    dir.extract(sources_dir.as_path())?;
    let mut files: Vec<String> = dir
        .files()
        .iter()
        .filter_map(|file| {
            let ext = file.path().extension();
            if let Some(ext) = ext {
                if ext == "move" {
                    Some(sources_dir.join(file.path()).display().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .filter_map(|file| {
            if !file.ends_with("spec.move") {
                Some(file)
            } else {
                None
            }
        })
        .collect();
    for sub_dir in dir.dirs() {
        let sub_files: Vec<String> = sub_dir
            .files()
            .iter()
            .filter_map(|file| {
                let ext = file.path().extension();
                if let Some(ext) = ext {
                    if ext == "move" {
                        Some(
                            sources_dir
                                .join(dir.path())
                                .join(file.path())
                                .display()
                                .to_string(),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter_map(|file| {
                if !file.ends_with("spec.move") {
                    Some(file)
                } else {
                    None
                }
            })
            .collect();
        files.extend(sub_files);
    }
    Ok(SourceFiles { temp_dir, files })
}
pub mod natives;
