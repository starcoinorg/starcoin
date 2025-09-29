/// The module provide block rewarding calculation logic.
module starcoin_framework::block_reward {

    use std::error;
    use std::option;
    use std::vector;
    use starcoin_framework::create_signer::create_signer;

    use starcoin_framework::object::{Self, Object};
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::fungible_asset;
    use starcoin_framework::fungible_asset::{FungibleAsset, FungibleStore, create_store};

    use starcoin_framework::account;
    use starcoin_framework::block_reward_config;
    use starcoin_framework::coin;
    use starcoin_framework::create_signer;
    use starcoin_framework::event;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::system_addresses;
    use starcoin_framework::treasury;
    use starcoin_framework::dao_treasury_withdraw_proposal;

    use starcoin_std::debug;

    /// Queue of rewards distributed to miners.
    struct RewardQueue has key {
        /// How many block rewards has been handled.
        reward_number: u64,
        /// informations about the reward distribution.
        infos: vector<RewardInfo>,
        /// event handle used to emit block reward event.
        reward_events: event::EventHandle<Self::BlockRewardEvent>,
        /// Gas fee store for every reward info
        gas_fees_store: Object<FungibleStore>,
        /// `gas_fees_store` Gas fee store owner address
        owner_address: address,
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
        gas_fee_amount: u64,
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
    const EREWARD_STC_FA_NOT_INITIALIZED: u64 = 106;

    /// Initialize the module, should be called in genesis.
    public fun initialize(framework: &signer, reward_delay: u64) {
        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(framework);

        let constructor_ref = object::create_named_object(framework, b"block_reward");
        let stc_metadata = coin::paired_metadata<STC>();
        assert!(option::is_some(&stc_metadata), error::invalid_state(EREWARD_STC_FA_NOT_INITIALIZED));

        let gas_fees_store = create_store(&constructor_ref, option::destroy_some(stc_metadata));

        block_reward_config::initialize(framework, reward_delay);
        move_to<RewardQueue>(framework, RewardQueue {
            reward_number: 0,
            infos: vector::empty(),
            reward_events: account::new_event_handle<Self::BlockRewardEvent>(framework),
            gas_fees_store,
            owner_address: object::address_from_constructor_ref(&constructor_ref),
        });
    }

    /// Process the given block rewards.
    public fun process_block_reward(
        account: &signer,
        current_number: u64,
        current_reward: u128,
        current_author: address,
        _auth_key_vec: vector<u8>,
        previous_block_gas_fees: FungibleAsset
    ) acquires RewardQueue {
        debug::print(&std::string::utf8(b"block_reward::process_block_reward | Entered"));

        system_addresses::assert_starcoin_framework(account);

        if (current_number == 0) {
            fungible_asset::destroy_zero(previous_block_gas_fees);
            debug::print(&std::string::utf8(b"block_reward::process_block_reward | Exited, current_number is 0"));
            return
        };

        let reward_queue = borrow_global_mut<RewardQueue>(system_addresses::get_starcoin_framework());
        let len = vector::length(&reward_queue.infos);

        debug::print(&std::string::utf8(b"block_reward::process_block_reward | rewards info len: "));
        debug::print(&len);

        assert!(
            (current_number == (reward_queue.reward_number + len + 1)),
            error::invalid_argument(ECURRENT_NUMBER_IS_WRONG)
        );

        // distribute gas fee to last block reward info.
        // if not last block reward info, the passed in gas fee must be zero.
        if (len == 0) {
            fungible_asset::destroy_zero(previous_block_gas_fees);
        } else {
            let reward_info = vector::borrow_mut(&mut reward_queue.infos, len - 1);
            assert!(current_number == reward_info.number + 1, error::invalid_argument(ECURRENT_NUMBER_IS_WRONG));
            reward_info.gas_fee_amount = reward_info.gas_fee_amount + fungible_asset::amount(&previous_block_gas_fees);
            fungible_asset::deposit(reward_queue.gas_fees_store, previous_block_gas_fees);
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
                    gas_fee_amount,
                    miner
                } = vector::remove(&mut reward_queue.infos, 0);

                let total_reward = gas_fee_amount;
                debug::print(&std::string::utf8(b"block_reward::process_block_reward | total_reward: "));
                debug::print(&gas_fee_amount);

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
                        let reward_stc = dao_treasury_withdraw_proposal::withdraw_for_block_reward<STC>(
                            account,
                            block_reward
                        );
                        // TODO(BobOng): To remove this convert after all module converting to fungible asset
                        fungible_asset::deposit(reward_queue.gas_fees_store, coin::coin_to_fungible_asset(reward_stc));
                    };
                };

                if (total_reward > 0) {
                    primary_fungible_store::deposit(
                        miner,
                        fungible_asset::withdraw(
                            &create_signer(reward_queue.owner_address),
                            reward_queue.gas_fees_store,
                            total_reward
                        )
                    );
                };
                debug::print(&std::string::utf8(b"block_reward::process_block_reward | before emit reward event"));

                // emit reward event.
                event::emit_event<BlockRewardEvent>(
                    &mut reward_queue.reward_events,
                    BlockRewardEvent {
                        block_number: reward_block_number,
                        block_reward,
                        gas_fees: (gas_fee_amount as u128),
                        miner,
                    }
                );

                debug::print(&std::string::utf8(b"block_reward::process_block_reward | after emit reward event"));

                reward_queue.reward_number = reward_queue.reward_number + 1;
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
            gas_fee_amount: 0
        };
        vector::push_back(&mut reward_queue.infos, current_info);

        debug::print(&std::string::utf8(b"block_reward::process_block_reward | Exited"));
    }
}