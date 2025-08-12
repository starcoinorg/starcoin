/// The module provide block rewarding calculation logic.
spec starcoin_framework::block_reward {
    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    spec initialize(account: &signer, reward_delay: u64) {
        use std::signer;
        use starcoin_framework::on_chain_config;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        include on_chain_config::PublishNewConfigAbortsIf<block_reward_config::RewardConfig>;
        include on_chain_config::PublishNewConfigEnsures<block_reward_config::RewardConfig>;
        aborts_if exists<RewardQueue>(system_addresses::get_starcoin_framework());
        ensures exists<RewardQueue>(system_addresses::get_starcoin_framework());
    }

    spec process_block_reward {
        use std::signer;
        use starcoin_framework::on_chain_config;
        use starcoin_framework::block_reward_config;

        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        // abort if current block is genesis, and previous block gas fees != 0
        aborts_if current_number == 0 && coin::value(previous_block_gas_fees) != 0;

        aborts_if current_number > 0 && !exists<RewardQueue>(system_addresses::get_starcoin_framework());
        aborts_if current_number > 0 && (global<RewardQueue>(
            system_addresses::get_starcoin_framework()
        ).reward_number + vector::length(
            global<RewardQueue>(system_addresses::get_starcoin_framework()).infos
        ) + 1) != current_number;
        aborts_if current_number > 0 && !exists<on_chain_config::Config<block_reward_config::RewardConfig>>(
            system_addresses::get_starcoin_framework()
        );


        let reward_info_length = vector::length(global<RewardQueue>(system_addresses::get_starcoin_framework()).infos);

        // abort if no previous block but has gas fees != 0.
        aborts_if current_number > 0 && reward_info_length == 0 && coin::value(previous_block_gas_fees) != 0;
        // abort if previous block number != current_block_number - 1.
        aborts_if current_number > 0 && reward_info_length != 0 && vector::borrow(
            global<RewardQueue>(system_addresses::get_starcoin_framework()).infos,
            reward_info_length - 1
        ).number != current_number - 1;

        aborts_if current_number > 0 && vector::length(
            global<RewardQueue>(system_addresses::get_starcoin_framework()).infos
        ) >= global<on_chain_config::Config<block_reward_config::RewardConfig>>(
            system_addresses::get_starcoin_framework()
        ).payload.reward_delay
            && (global<RewardQueue>(system_addresses::get_starcoin_framework()).reward_number + 1) != vector::borrow(
            global<RewardQueue>(system_addresses::get_starcoin_framework()).infos,
            0
        ).number;

        aborts_if current_number > 0 && vector::length(
            global<RewardQueue>(system_addresses::get_starcoin_framework()).infos
        ) >= global<on_chain_config::Config<block_reward_config::RewardConfig>>(
            system_addresses::get_starcoin_framework()
        ).payload.reward_delay
            && (global<RewardQueue>(system_addresses::get_starcoin_framework()).reward_number + 1) > max_u64();

        aborts_if current_number > 0 && !account::exists_at(current_author) ;

        pragma verify = false;
    }
}