script {
use 0x1::VMConfig;
use 0x1::RewardConfig;
use 0x1::Version;
use 0x1::Config;
use 0x1::Coin;
use 0x1::Consensus;

//TODO refactor when move support ABI, and pass struct by argument
fun config_init(config_account: &signer,
    publishing_option: vector<u8>, instruction_schedule: vector<u8>,native_schedule: vector<u8>,
    reward_halving_interval: u64, reward_base: u64, reward_delay: u64,
    uncle_rate_target:u64,epoch_time_target: u64,reward_half_time_target: u64) {

    Config::initialize(config_account);

    // Currency setup
    Coin::initialize(config_account);

    VMConfig::initialize(config_account, publishing_option, instruction_schedule, native_schedule);
    RewardConfig::initialize(config_account, reward_halving_interval, reward_base, reward_delay);
    Version::initialize(config_account);
    Consensus::initialize(config_account,uncle_rate_target,epoch_time_target,reward_half_time_target);
}
}
