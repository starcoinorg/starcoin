pub mod miner;
mod stratum;
#[cfg(test)]
mod test;
mod worker;

use byteorder::{LittleEndian, WriteBytesExt};
use rand::Rng;
use std::ops::Range;

fn partition_nonce(id: u64, total: u64) -> Range<u64> {
    let span = u64::max_value() / total;
    let start = span * id;
    let end = match id {
        x if x < total - 1 => start + span,
        x if x == total - 1 => u64::max_value(),
        _ => unreachable!(),
    };
    Range { start, end }
}

fn nonce_generator(range: Range<u64>) -> impl FnMut() -> u64 {
    let mut rng = rand::thread_rng();
    let Range { start, end } = range;
    move || rng.gen_range(start, end)
}

pub fn set_header_nonce(header: &[u8], nonce: u64) -> Vec<u8> {
    let len = header.len();
    let mut header = header.to_owned();
    header.truncate(len - 8);
    let _ = header.write_u64::<LittleEndian>(nonce);
    header
}
