// ref aptos-core/aptos-move/framework/src/lib.rs
pub mod natives;

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::{
    io::{Read, Write},
    path::PathBuf,
};

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative.into())
}

pub(crate) fn path_relative_to_crate(path: PathBuf) -> PathBuf {
    let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(crate_path).unwrap_or(&path).to_path_buf()
}

pub fn zip_metadata(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::best());
    e.write_all(data)?;
    Ok(e.finish()?)
}

pub fn zip_metadata_str(s: &str) -> anyhow::Result<Vec<u8>> {
    zip_metadata(s.as_bytes())
}

pub fn unzip_metadata(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut d = GzDecoder::new(data);
    let mut res = vec![];
    d.read_to_end(&mut res)?;
    Ok(res)
}

pub fn unzip_metadata_str(data: &[u8]) -> anyhow::Result<String> {
    let r = unzip_metadata(data)?;
    let s = String::from_utf8(r)?;
    Ok(s)
}
