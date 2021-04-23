use starcoin_types::U256;
pub mod rpc;
pub mod service;
pub mod stratum;

pub fn difficulty_to_target_hex(difficulty: U256) -> String {
    let target = format!("{:x}", U256::from(u64::max_value()) / difficulty);
    let mut temp = "0".repeat(16 - target.len());
    temp.push_str(&target);
    let mut t = hex::decode(temp).expect("Decode target never failed");
    t.reverse();
    hex::encode(&t)
}
