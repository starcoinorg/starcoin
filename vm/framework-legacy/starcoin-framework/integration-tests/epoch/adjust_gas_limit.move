//# init -n dev

//# faucet --addr Genesis

//# run --signers Genesis
// test adjust epoch zero uncle.
script {
use StarcoinFramework::ConsensusConfig;
use StarcoinFramework::Epoch;
use StarcoinFramework::Option;
    use StarcoinFramework::Debug;

fun main() {
    let config = ConsensusConfig::get_config();
    let block_gas_limit: u64 = 2000000000;
    let block_count = ConsensusConfig::epoch_block_count(&config);

    // min
    let min_time_target = ConsensusConfig::min_block_time_target(&config);
    let new_gas_limit_1 = Epoch::compute_gas_limit(&config, min_time_target, min_time_target, block_gas_limit, (block_gas_limit * block_count as u128));
    let base_gas_limit = ConsensusConfig::base_block_gas_limit(&config);
    assert!(Option::is_some(&new_gas_limit_1), 100);
    let new_gas_limit_1 = Option::destroy_some(new_gas_limit_1);
    Debug::print<u64>(&base_gas_limit);
    Debug::print<u64>(&new_gas_limit_1);
    assert!(new_gas_limit_1 > base_gas_limit, 101);
    assert!(new_gas_limit_1 > block_gas_limit, 106);

    // max
    let max_time_target = ConsensusConfig::max_block_time_target(&config);
    let new_gas_limit_2 = Epoch::compute_gas_limit(&config, max_time_target, max_time_target, block_gas_limit, (block_gas_limit * block_count / 2 as u128));
    assert!(Option::is_some(&new_gas_limit_2), 102);
    let new_gas_limit_2 = Option::destroy_some(new_gas_limit_2);
    Debug::print<u64>(&new_gas_limit_2);
    assert!(new_gas_limit_2 > base_gas_limit, 103);
    assert!(new_gas_limit_2 < block_gas_limit, 104);

    // other
    let new_gas_limit_3 = Epoch::compute_gas_limit(&config, 40, 40, block_gas_limit, (block_gas_limit * block_count / 2 as u128));
    assert!(Option::is_none(&new_gas_limit_3), 105);
}
}


// check: EXECUTED

