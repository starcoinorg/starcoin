use starcoin_types::U256;

pub mod codec;
pub mod diff_manager;
pub mod pplns_store;
pub mod rpc;

pub fn difficulty_to_target_hex(difficulty: U256) -> String {
    let target = format!("{:x}", U256::from(u64::MAX) / difficulty);
    let mut temp = "0".repeat(16 - target.len());
    temp.push_str(&target);
    let mut t = hex::decode(temp).expect("Decode target never failed");
    t.reverse();
    hex::encode(&t)
}

pub fn target_hex_to_difficulty(target: &str) -> anyhow::Result<U256> {
    let mut temp = hex::decode(target)?;
    temp.reverse();
    let temp = hex::encode(temp);
    let temp = U256::from_str_radix(&temp, 16)?;
    Ok(U256::from(u64::MAX) / temp)
}
