address 0x1 {

module BlockReward {
    use 0x1::Timestamp;
    use 0x1::Token::{Self,Coin};
    use 0x1::STC::{STC};
    use 0x1::Vector;
    use 0x1::Account;
    use 0x1::RewardConfig;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    resource struct BlockReward{
        balance: Coin<STC>,
    }

    resource struct RewardInfo {
        reward_height: u64,
        heights: vector<u64>,
        miners: vector<address>,
    }

    public fun initialize(account: &signer, reward_balance: u64) {
        assert(Timestamp::is_genesis(), 1);
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        move_to<RewardInfo>(account, RewardInfo {
            reward_height: 0,
            heights: Vector::empty(),
            miners: Vector::empty(),
        });
        let reward_token = Token::mint<STC>(account,  reward_balance);
        move_to<BlockReward>(account, BlockReward {
            balance: reward_token,
        });
    }

    fun withdraw(amount: u64): Coin<STC> acquires BlockReward {
        let block_reward = borrow_global_mut<BlockReward>(CoreAddresses::GENESIS_ACCOUNT());
        Token::withdraw<STC>(&mut block_reward.balance, amount)
    }

    public fun process_block_reward(account: &signer, current_height: u64, current_author: address, auth_key_prefix: vector<u8>) acquires RewardInfo, BlockReward {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

        if (current_height > 0) {
            let reward_info = borrow_global_mut<RewardInfo>(CoreAddresses::GENESIS_ACCOUNT());
            let len = Vector::length(&reward_info.heights);
            let miner_len = Vector::length(&reward_info.miners);
            assert((current_height == (reward_info.reward_height + len + 1)), 6002);
            assert((len <= RewardConfig::reward_delay()), 6003);
            assert((len == miner_len), 6004);

            if (len == RewardConfig::reward_delay()) {//pay and remove
                let reward_height = *&reward_info.reward_height + 1;
                let first_height = *Vector::borrow(&reward_info.heights, 0);
                assert((reward_height == first_height), 6005);

                let reward_amount = RewardConfig::reward_coin(reward_height);
                let reward_miner = *Vector::borrow(&reward_info.miners, 0);
                reward_info.reward_height = reward_height;
                if (reward_amount > 0) {
                    assert(Account::exists_at(reward_miner), 6006);
                    let reward = withdraw(reward_amount);
                    Account::deposit<STC>(account, reward_miner, reward);
                };
                Vector::remove(&mut reward_info.heights, 0);
                Vector::remove(&mut reward_info.miners, 0);
            };

            Vector::push_back(&mut reward_info.heights, current_height);
            if (!Account::exists_at(current_author)) {
                assert(!Vector::is_empty(&auth_key_prefix), 6007);
                Account::create_account<STC>(current_author, auth_key_prefix);
            };
            Vector::push_back(&mut reward_info.miners, current_author);
        };
    }
}
}