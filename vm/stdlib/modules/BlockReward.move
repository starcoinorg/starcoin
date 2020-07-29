address 0x1 {

module BlockReward {
    use 0x1::Timestamp;
    use 0x1::Token::{Self,Token};
    use 0x1::STC::{STC};
    use 0x1::Vector;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::ErrorCode;

    resource struct BlockReward{
        balance: Token<STC>,
    }

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

    public fun initialize(account: &signer, reward_balance: u128, reward_delay: u64) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        assert(reward_delay > 0, 4);
        move_to<RewardQueue>(account, RewardQueue {
            reward_number: 0,
            reward_delay: reward_delay,
            infos: Vector::empty(),
        });
        let reward_token = Token::mint<STC>(account,  reward_balance);
        move_to<BlockReward>(account, BlockReward {
            balance: reward_token,
        });
    }

    fun withdraw(amount: u128): Token<STC> acquires BlockReward {
        let block_reward = borrow_global_mut<BlockReward>(CoreAddresses::GENESIS_ACCOUNT());
        let real_amount = if (Token::value<STC>(&block_reward.balance) < amount) {
            Token::value<STC>(&block_reward.balance)
        } else {
            amount
        };
        Token::withdraw<STC>(&mut block_reward.balance, real_amount)
    }

    public fun process_block_reward(account: &signer, current_number: u64, current_reward: u128,
        current_author: address, auth_key_prefix: vector<u8>) acquires RewardQueue, BlockReward {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        if (current_number > 0) {
            let rewards = borrow_global_mut<RewardQueue>(CoreAddresses::GENESIS_ACCOUNT());
            let len = Vector::length(&rewards.infos);
            assert((current_number == (rewards.reward_number + len + 1)), 6002);
            assert(len <= rewards.reward_delay, 6003);

            if (len == rewards.reward_delay) {//pay and remove
                let reward_number = *&rewards.reward_number + 1;
                let first_info = *Vector::borrow(&rewards.infos, 0);
                assert((reward_number == first_info.number), 6005);

                rewards.reward_number = reward_number;
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
                number: current_number,
                reward: current_reward,
                miner: current_author,
            };
            Vector::push_back(&mut rewards.infos, current_info);
        };
    }
}
}