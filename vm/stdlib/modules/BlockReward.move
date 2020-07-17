address 0x1 {

module BlockReward {
    use 0x1::Timestamp;
    use 0x1::Token::{Self,Token};
    use 0x1::STC::{STC};
    use 0x1::Vector;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    resource struct BlockReward{
        balance: Token<STC>,
    }

    resource struct RewardQueue {
        reward_height: u64,
        reward_delay: u64,
        infos: vector<RewardInfo>,
    }

    struct RewardInfo {
        height: u64,
        reward: u64,
        miner: address,
    }

    public fun initialize(account: &signer, reward_balance: u64, reward_delay: u64) {
        assert(Timestamp::is_genesis(), 1);
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        assert(reward_delay > 0, 4);
        move_to<RewardQueue>(account, RewardQueue {
            reward_height: 0,
            reward_delay: reward_delay,
            infos: Vector::empty(),
        });
        let reward_token = Token::mint<STC>(account,  reward_balance);
        move_to<BlockReward>(account, BlockReward {
            balance: reward_token,
        });
    }

    fun withdraw(amount: u64): Token<STC> acquires BlockReward {
        let block_reward = borrow_global_mut<BlockReward>(CoreAddresses::GENESIS_ACCOUNT());
        Token::withdraw<STC>(&mut block_reward.balance, amount)
    }

    public fun process_block_reward(account: &signer, current_height: u64, current_reward: u64,
        current_author: address, auth_key_prefix: vector<u8>) acquires RewardQueue, BlockReward {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

        if (current_height > 0) {
            let rewards = borrow_global_mut<RewardQueue>(CoreAddresses::GENESIS_ACCOUNT());
            let len = Vector::length(&rewards.infos);
            assert((current_height == (rewards.reward_height + len + 1)), 6002);
            assert(len <= rewards.reward_delay, 6003);

            if (len == rewards.reward_delay) {//pay and remove
                let reward_height = *&rewards.reward_height + 1;
                let first_info = *Vector::borrow(&rewards.infos, 0);
                assert((reward_height == first_info.height), 6005);

                rewards.reward_height = reward_height;
                if (first_info.reward > 0) {
                    assert(Account::exists_at(first_info.miner), 6006);
                    let reward = Self::withdraw(first_info.reward);
                    Account::deposit<STC>(account, first_info.miner, reward);
                };
                Vector::remove(&mut rewards.infos, 0);
            };

            if (!Account::exists_at(current_author)) {
                assert(!Vector::is_empty(&auth_key_prefix), 6007);
                Account::create_account<STC>(current_author, auth_key_prefix);
            };
            let current_info = RewardInfo {
                height: current_height,
                reward: current_reward,
                miner: current_author,
            };
            Vector::push_back(&mut rewards.infos, current_info);
        };
    }
}
}