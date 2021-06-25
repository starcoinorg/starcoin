use starcoin_types::U256;

pub mod rpc;
pub mod service;
pub mod stratum;
pub use crate::rpc::gen_client::Client as StratumRpcClient;
pub use anyhow::Result;

pub fn difficulty_to_target_hex(difficulty: U256) -> String {
    let target = format!("{:x}", U256::from(u64::max_value()) / difficulty);
    let mut temp = "0".repeat(16 - target.len());
    temp.push_str(&target);
    let mut t = hex::decode(temp).expect("Decode target never failed");
    t.reverse();
    hex::encode(&t)
}

pub fn target_hex_to_difficulty(target: &str) -> Result<U256> {
    let mut temp = hex::decode(target)?;
    temp.reverse();
    let temp = hex::encode(temp);
    let temp = U256::from_str_radix(&temp, 16)?;
    Ok(U256::from(u64::max_value()) / temp)
}

#[test]
fn test() {
    let target = difficulty_to_target_hex(1024.into());
    let diff = target_hex_to_difficulty(&target).unwrap();
    println!("{}", diff);
}
