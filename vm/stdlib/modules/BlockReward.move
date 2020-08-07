address 0x1 {

module BlockReward {
    use 0x1::Timestamp;
    use 0x1::Token;
    use 0x1::STC::{STC};
    use 0x1::Vector;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::ErrorCode;

    resource struct RewardQueue {
        reward_number: u64,
        reward_delay: u64,
        infos: vector<RewardInfo>,
    }

    struct RewardInfo {
        number: u64,
        reward: u128,
        miner: address,
    }

    fun AUTH_KEY_PREFIX_IS_NOT_EMPTY(): u64 { ErrorCode::ECODE_BASE() + 1}
    fun CURRENT_NUMBER_IS_WRONG(): u64 { ErrorCode::ECODE_BASE() + 2}
    fun LEN_OF_REWARD_INFO_IS_WRONG(): u64 { ErrorCode::ECODE_BASE() + 3}
    fun REWARD_NUMBER_IS_WRONG(): u64 { ErrorCode::ECODE_BASE() + 4}
    fun MINER_EXIST(): u64 { ErrorCode::ECODE_BASE() + 5}

    public fun initialize(account: &signer, reward_delay: u64) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        assert(reward_delay > 0, ErrorCode::EINVALID_ARGUMENT());
        move_to<RewardQueue>(account, RewardQueue {
            reward_number: 0,
            reward_delay: reward_delay,
            infos: Vector::empty(),
        });
    }

    public fun process_block_reward(account: &signer, current_number: u64, current_reward: u128,
        current_author: address, auth_key_prefix: vector<u8>) acquires RewardQueue {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        if (current_number > 0) {
            let rewards = borrow_global_mut<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
            let len = Vector::length(&rewards.infos);
            assert((current_number == (rewards.reward_number + len + 1)), CURRENT_NUMBER_IS_WRONG());
            assert(len <= rewards.reward_delay, LEN_OF_REWARD_INFO_IS_WRONG());

            if (len == rewards.reward_delay) {//pay and remove
                let reward_number = *&rewards.reward_number + 1;
                let first_info = *Vector::borrow(&rewards.infos, 0);
                assert((reward_number == first_info.number), REWARD_NUMBER_IS_WRONG());

                rewards.reward_number = reward_number;
                if (first_info.reward > 0) {
                    assert(Account::exists_at(first_info.miner), MINER_EXIST());
                    let reward = Token::mint<STC>(account, first_info.reward);
                    Account::deposit_to<STC>(account, first_info.miner, reward);
                };
                Vector::remove(&mut rewards.infos, 0);
            };

            if (!Account::exists_at(current_author)) {
                assert(!Vector::is_empty(&auth_key_prefix), AUTH_KEY_PREFIX_IS_NOT_EMPTY());
                Account::create_account<STC>(current_author, auth_key_prefix);
            };
            let current_info = RewardInfo {
                number: current_number,
                reward: current_reward,
                miner: current_author,
            };
            Vector::push_back(&mut rewards.infos, current_info);
        };
    }
}
}