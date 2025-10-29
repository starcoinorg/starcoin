/// The module provide block rewarding calculation logic.
module starcoin_framework::block_reward {

    use std::error;
    use std::string::utf8;
    use std::vector;
    use starcoin_framework::primary_fungible_store;

    use starcoin_framework::account;
    use starcoin_framework::block_reward_config;
    use starcoin_framework::coin;
    use starcoin_framework::create_signer;
    use starcoin_framework::create_signer::create_signer;
    use starcoin_framework::dao_treasury_withdraw_proposal;
    use starcoin_framework::event;
    use starcoin_framework::fungible_asset::{Self, create_store, FungibleAsset, FungibleStore};
    use starcoin_framework::object::{Self, Object};
    use starcoin_framework::starcoin_coin::{Self, STC};
    use starcoin_framework::system_addresses::{Self, get_starcoin_framework};
    use starcoin_framework::treasury;
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
    }

    /// Reward info of miners.
    struct RewardInfo has store {
        /// number of the block miner minted.
        number: u64,
        /// how many stc rewards.
        block_reward_amount: u128,
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
        debug::print(&utf8(b"block_reward::initialize | Entered "));

        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(framework);

        let constructor_ref = object::create_named_object(framework, b"block_reward");

        block_reward_config::initialize(framework, reward_delay);
        move_to<RewardQueue>(framework, RewardQueue {
            reward_number: 0,
            infos: vector::empty(),
            reward_events: account::new_event_handle<Self::BlockRewardEvent>(framework),
            gas_fees_store: create_store(&constructor_ref, starcoin_coin::get_stc_fa_metadata()),
        });

        debug::print(&utf8(b"block_reward::initialize | Exited"));
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
        system_addresses::assert_starcoin_framework(account);

        if (current_number == 0) {
            fungible_asset::destroy_zero(previous_block_gas_fees);
            return
        };

        let reward_queue = borrow_global_mut<RewardQueue>(system_addresses::get_starcoin_framework());
        let len = vector::length(&reward_queue.infos);

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
        if (len >= reward_delay) {
            // pay and remove
            let i = len;
            while (i > 0 && i >= reward_delay) {
                let RewardInfo {
                    number: reward_block_number,
                    block_reward_amount,
                    gas_fee_amount,
                    miner
                } = vector::remove(&mut reward_queue.infos, 0);

                let total_reward_fa = fungible_asset::zero(starcoin_coin::get_stc_fa_metadata());

                // Add block reward to total.
                if (block_reward_amount > 0) {
                    // if no STC in Treasury, BlockReward will been 0.
                    let treasury_balance = treasury::balance<STC>(get_starcoin_framework());
                    if (treasury_balance < block_reward_amount) {
                        block_reward_amount = treasury_balance;
                    };

                    let reward_stc = dao_treasury_withdraw_proposal::withdraw_for_block_reward<STC>(
                        account,
                        block_reward_amount
                    );
                    fungible_asset::merge(&mut total_reward_fa, reward_stc);
                };

                // Process gas fee reward
                if (gas_fee_amount > 0) {
                    debug::print(&utf8(b"block_reward::process_block_reward | gas fee amount: "));
                    let gas_fee_fa = fungible_asset::withdraw(
                        &create_signer(object::owner(reward_queue.gas_fees_store)),
                        reward_queue.gas_fees_store,
                        gas_fee_amount
                    );
                    fungible_asset::merge(&mut total_reward_fa, gas_fee_fa);
                };

                if (fungible_asset::amount(&total_reward_fa) > 0) {
                    primary_fungible_store::deposit(miner, total_reward_fa);
                } else {
                    fungible_asset::destroy_zero(total_reward_fa);
                };

                debug::print(&utf8(b"block_reward::process_block_reward | finish process block number: "));
                debug::print(&reward_block_number);
                debug::print(&(block_reward_amount + (gas_fee_amount as u128)));

                // emit reward event.
                event::emit_event<BlockRewardEvent>(
                    &mut reward_queue.reward_events,
                    BlockRewardEvent {
                        block_number: reward_block_number,
                        block_reward: block_reward_amount,
                        gas_fees: (gas_fee_amount as u128),
                        miner,
                    }
                );
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
            block_reward_amount: current_reward,
            miner: current_author,
            gas_fee_amount: 0
        };
        vector::push_back(&mut reward_queue.infos, current_info);
    }

    #[test(framework = @0x1, alice = @0x12345)]
    fun block_reward_basic_test(framework: &signer, alice: &signer) acquires RewardQueue {
        use std::signer;
        use starcoin_framework::starcoin_account;

        // Do initliazed for enviroment
        let (burn_cap, mint_cap) = starcoin_coin::initialize_for_test(framework);
        starcoin_coin::ensure_initialized_with_stc_fa_metadata_for_test();
        starcoin_account::create_account(signer::address_of(alice));
        account::create_account_for_test(signer::address_of(framework));
        let cap = treasury::initialize<STC>(framework, starcoin_coin::mint_stc_fa_for_test(1000000000000));
        dao_treasury_withdraw_proposal::plugin(framework, cap);

        Self::initialize(framework, 1);

        let metadata = starcoin_coin::get_stc_fa_metadata();
        let alice = signer::address_of(alice);

        // Block 1, last reward is 0, current reward is 10000, gas fee is 0
        Self::process_block_reward(
            framework,
            1,
            10000,
            alice,
            vector::empty<u8>(),
            fungible_asset::zero(metadata)
        );
        let balance = primary_fungible_store::balance(alice, metadata);
        debug::print(&balance);
        assert!(primary_fungible_store::balance(alice, metadata) == 0, 1);

        // Block 2, last reward is 10000, current reward is 10000, gas fee is zero
        Self::process_block_reward(
            framework,
            2,
            10000,
            alice,
            vector::empty<u8>(),
            fungible_asset::zero(metadata)
        );
        let balance = primary_fungible_store::balance(alice, metadata);
        debug::print(&balance);
        assert!(balance == 10000, 2);

        // Block 3, last reward is 10000, current reward is 10000, gas fee is 5000
        Self::process_block_reward(
            framework,
            3,
            10000,
            alice,
            vector::empty<u8>(),
            starcoin_coin::mint_stc_fa_for_test(10000),
        );
        let balance = primary_fungible_store::balance(alice, metadata);
        debug::print(&balance);
        assert!(balance == 30000, 3);

        // Block 4, last reward is 10000, current reward is 0, gas fee is 0
        Self::process_block_reward(
            framework,
            4,
            0,
            alice,
            vector::empty<u8>(),
            fungible_asset::zero(metadata),
        );
        let balance = primary_fungible_store::balance(alice, metadata);
        debug::print(&balance);
        assert!(balance == 40000, 4);

        coin::destroy_mint_cap(mint_cap);
        coin::destroy_burn_cap(burn_cap);
    }
}