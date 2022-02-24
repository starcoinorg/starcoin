// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use include_dir::{include_dir, Dir};
use log::info;
use once_cell::sync::Lazy;
use tempfile::TempDir;

pub const SOURCES_DIR: Dir = include_dir!("sources");

#[derive(Debug)]
pub struct SourceFiles {
    pub tempdir: TempDir,
    pub files: Vec<String>,
}

pub static STARCOIN_FRAMEWORK_SOURCES: Lazy<SourceFiles> =
    Lazy::new(|| restore_sources().expect("Restore source file error"));

//restore the sources files to a tempdir.
fn restore_sources() -> anyhow::Result<SourceFiles> {
    let tempdir = tempfile::tempdir()?;
    let sources_dir = tempdir.path().join("starcoin-framework").join("sources");
    info!("restore starcoin-framework sources in: {:?}", sources_dir);
    std::fs::create_dir_all(sources_dir.as_path())?;
    SOURCES_DIR.extract(sources_dir.as_path())?;
    let files = SOURCES_DIR
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
        .collect();
    Ok(SourceFiles { tempdir, files })
}
