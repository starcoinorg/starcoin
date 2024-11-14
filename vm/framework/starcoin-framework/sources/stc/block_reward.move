/// The module provide block rewarding calculation logic.
module starcoin_framework::block_reward {

    use std::error;
    use std::vector;

    use starcoin_framework::account;
    use starcoin_framework::block_reward_config;
    use starcoin_framework::coin;
    use starcoin_framework::create_signer;
    use starcoin_framework::event;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::system_addresses;
    use starcoin_framework::treasury;
    use starcoin_framework::treasury_withdraw_dao_proposal;
    use starcoin_std::debug;

    /// Queue of rewards distributed to miners.
    struct RewardQueue has key {
        /// How many block rewards has been handled.
        reward_number: u64,
        /// informations about the reward distribution.
        infos: vector<RewardInfo>,
        /// event handle used to emit block reward event.
        reward_events: event::EventHandle<Self::BlockRewardEvent>,
    }

    /// Reward info of miners.
    struct RewardInfo has store {
        /// number of the block miner minted.
        number: u64,
        /// how many stc rewards.
        reward: u128,
        /// miner who mint the block.
        miner: address,
        /// store the gas fee that users consumed.
        gas_fees: coin::Coin<STC>,
    }

    /// block reward event
    struct BlockRewardEvent has drop, store {
        /// block number
        block_number: u64,
        /// STC reward.
        block_reward: u128,
        /// gas fees in STC.
        gas_fees: u128,
        /// block miner
        miner: address,
    }

    const EAUTHOR_AUTH_KEY_IS_EMPTY: u64 = 101;
    const ECURRENT_NUMBER_IS_WRONG: u64 = 102;
    const EREWARD_NUMBER_IS_WRONG: u64 = 103;
    const EMINER_EXIST: u64 = 104;
    const EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 105;

    /// Initialize the module, should be called in genesis.
    public fun initialize(account: &signer, reward_delay: u64) {
        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(account);

        block_reward_config::initialize(account, reward_delay);
        move_to<RewardQueue>(account, RewardQueue {
            reward_number: 0,
            infos: vector::empty(),
            reward_events: account::new_event_handle<Self::BlockRewardEvent>(account),
        });
    }

    /// Process the given block rewards.
    public fun process_block_reward(
        account: &signer,
        current_number: u64,
        current_reward: u128,
        current_author: address,
        _auth_key_vec: vector<u8>,
        previous_block_gas_fees: coin::Coin<STC>
    ) acquires RewardQueue {
        debug::print(&std::string::utf8(b"block_reward::process_block_reward | Entered"));

        system_addresses::assert_starcoin_framework(account);

        if (current_number == 0) {
            coin::destroy_zero(previous_block_gas_fees);
            debug::print(&std::string::utf8(b"block_reward::process_block_reward | Exited, current_number is 0"));
            return
        };

        let rewards = borrow_global_mut<RewardQueue>(system_addresses::get_starcoin_framework());
        let len = vector::length(&rewards.infos);

        debug::print(&std::string::utf8(b"block_reward::process_block_reward | rewards info len: "));
        debug::print(&len);

        assert!(
            (current_number == (rewards.reward_number + len + 1)),
            error::invalid_argument(ECURRENT_NUMBER_IS_WRONG)
        );

        // distribute gas fee to last block reward info.
        // if not last block reward info, the passed in gas fee must be zero.
        if (len == 0) {
            coin::destroy_zero(previous_block_gas_fees);
        } else {
            let reward_info = vector::borrow_mut(&mut rewards.infos, len - 1);
            assert!(current_number == reward_info.number + 1, error::invalid_argument(ECURRENT_NUMBER_IS_WRONG));
            coin::merge(&mut reward_info.gas_fees, previous_block_gas_fees);
        };

        let reward_delay = block_reward_config::reward_delay();
        debug::print(&std::string::utf8(b"block_reward::process_block_reward | rewards delay: "));
        debug::print(&reward_delay);
        if (len >= reward_delay) {
            //pay and remove
            let i = len;
            while (i > 0 && i >= reward_delay) {
                let RewardInfo {
                    number: reward_block_number,
                    reward: block_reward,
                    gas_fees,
                    miner
                } = vector::remove(
                    &mut rewards.infos,
                    0
                );

                let gas_fee_value = (coin::value(&gas_fees) as u128);
                let total_reward = gas_fees;
                debug::print(&std::string::utf8(b"block_reward::process_block_reward | total_reward: "));
                debug::print(&coin::value(&total_reward));

                // add block reward to total.
                if (block_reward > 0) {
                    // if no STC in Treasury, BlockReward will been 0.
                    let treasury_balance = treasury::balance<STC>();
                    if (treasury_balance < block_reward) {
                        block_reward = treasury_balance;
                    };
                    debug::print(&std::string::utf8(b"block_reward::process_block_reward | treasury_balance: "));
                    debug::print(&treasury_balance);
                    if (block_reward > 0) {
                        let reward = treasury_withdraw_dao_proposal::withdraw_for_block_reward<STC>(account, block_reward);
                        coin::merge(&mut total_reward, reward);
                    };
                };

                // distribute total.
                debug::print(&std::string::utf8(b"block_reward::process_block_reward | distribute total reward: "));
                debug::print(&coin::value(&total_reward));
                debug::print(&miner);

                if (coin::value(&total_reward) > 0) {
                    coin::deposit<STC>(miner, total_reward);
                } else {
                    coin::destroy_zero(total_reward);
                };

                debug::print(&std::string::utf8(b"block_reward::process_block_reward | before emit reward event"));

                // emit reward event.
                event::emit_event<BlockRewardEvent>(
                    &mut rewards.reward_events,
                    BlockRewardEvent {
                        block_number: reward_block_number,
                        block_reward,
                        gas_fees: gas_fee_value,
                        miner,
                    }
                );

                debug::print(&std::string::utf8(b"block_reward::process_block_reward | after emit reward event"));

                rewards.reward_number = rewards.reward_number + 1;
                i = i - 1;
            }
        };

        account::create_account_if_does_not_exist(current_author);
        if (!coin::is_account_registered<STC>(current_author)) {
            coin::register<STC>(&create_signer::create_signer(current_author));
        };

        let current_info = RewardInfo {
            number: current_number,
            reward: current_reward,
            miner: current_author,
            gas_fees: coin::zero<STC>(),
        };
        vector::push_back(&mut rewards.infos, current_info);

        debug::print(&std::string::utf8(b"block_reward::process_block_reward | Exited"));
    }
}