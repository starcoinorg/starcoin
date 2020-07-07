script {
use 0x1::Genesis;

fun genesis_init(publishing_option: vector<u8>, instruction_schedule: vector<u8>,native_schedule: vector<u8>,
                                              reward_halving_interval: u64, reward_base: u64, reward_delay: u64,
                                              uncle_rate_target:u64,epoch_time_target: u64,reward_half_time_target: u64,
                                              total_supply: u64, pre_mine_percent:u64,
                                              parent_hash: vector<u8>,
                                              association_auth_key: vector<u8>,
                                              genesis_auth_key: vector<u8>,
                                              ) {
    Genesis::initialize(publishing_option, instruction_schedule,native_schedule,
                        reward_halving_interval, reward_base, reward_delay,
                        uncle_rate_target ,epoch_time_target,reward_half_time_target,
                        total_supply, pre_mine_percent,
                        parent_hash ,
                        association_auth_key, genesis_auth_key);
}
}