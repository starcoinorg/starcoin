address StarcoinFramework {
/// The module provide block rewarding calculation logic.
module BlockReward {
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Token;
    use StarcoinFramework::STC::{STC};
    use StarcoinFramework::Vector;
    use StarcoinFramework::Account;
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Errors;
    use StarcoinFramework::RewardConfig;
    use StarcoinFramework::Config;
    use StarcoinFramework::Event;
    use StarcoinFramework::Treasury;
    use StarcoinFramework::TreasuryWithdrawDaoProposal;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    /// Queue of rewards distributed to miners.
    struct RewardQueue has key {
        /// How many block rewards has been handled.
        reward_number: u64,
        /// informations about the reward distribution.
        infos: vector<RewardInfo>,
        /// event handle used to emit block reward event.
        reward_events: Event::EventHandle<Self::BlockRewardEvent>,
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
        gas_fees: Token::Token<STC>,
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
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);

        RewardConfig::initialize(account, reward_delay);
        move_to<RewardQueue>(account, RewardQueue {
            reward_number: 0,
            infos: Vector::empty(),
            reward_events: Event::new_event_handle<Self::BlockRewardEvent>(account),
        });
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::GENESIS_ADDRESS();
        include Config::PublishNewConfigAbortsIf<RewardConfig::RewardConfig>;
        include Config::PublishNewConfigEnsures<RewardConfig::RewardConfig>;
        aborts_if exists<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
        ensures exists<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
    }

    /// Process the given block rewards.
    public fun process_block_reward(account: &signer, current_number: u64, current_reward: u128,
                                    current_author: address, _auth_key_vec: vector<u8>,
                                    previous_block_gas_fees: Token::Token<STC>) acquires RewardQueue {
        CoreAddresses::assert_genesis_address(account);
        if (current_number == 0) {
            Token::destroy_zero(previous_block_gas_fees);
            return
        };

        let rewards = borrow_global_mut<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
        let len = Vector::length(&rewards.infos);
        assert!((current_number == (rewards.reward_number + len + 1)), Errors::invalid_argument(ECURRENT_NUMBER_IS_WRONG));

        // distribute gas fee to last block reward info.
        // if not last block reward info, the passed in gas fee must be zero.
        if (len == 0) {
            Token::destroy_zero(previous_block_gas_fees);
        } else {
            let reward_info = Vector::borrow_mut(&mut rewards.infos, len - 1);
            assert!(current_number == reward_info.number + 1, Errors::invalid_argument(ECURRENT_NUMBER_IS_WRONG));
            Token::deposit(&mut reward_info.gas_fees, previous_block_gas_fees);
        };

        let reward_delay = RewardConfig::reward_delay();
        if (len >= reward_delay) {//pay and remove
            let i = len;
            while (i > 0 && i >= reward_delay) {
                let RewardInfo { number: reward_block_number, reward: block_reward, gas_fees, miner } = Vector::remove(&mut rewards.infos, 0);

                let gas_fee_value = Token::value(&gas_fees);
                let total_reward = gas_fees;
                // add block reward to total.
                if (block_reward > 0) {
                    // if no STC in Treasury, BlockReward will been 0.
                    let treasury_balance = Treasury::balance<STC>();
                    if (treasury_balance < block_reward) {
                        block_reward = treasury_balance;
                    };
                    if (block_reward > 0) {
                        let reward = TreasuryWithdrawDaoProposal::withdraw_for_block_reward<STC>(account, block_reward);
                        Token::deposit(&mut total_reward, reward);
                    };
                };
                // distribute total.
                if (Token::value(&total_reward) > 0) {
                    Account::deposit<STC>(miner, total_reward);
                } else {
                    Token::destroy_zero(total_reward);
                };
                // emit reward event.
                Event::emit_event<BlockRewardEvent>(
                    &mut rewards.reward_events,
                    BlockRewardEvent {
                        block_number: reward_block_number,
                        block_reward: block_reward,
                        gas_fees: gas_fee_value,
                        miner,
                    }
                );

                rewards.reward_number = rewards.reward_number + 1;
                i = i - 1;
            }
        };

        if (!Account::exists_at(current_author)) {
            Account::create_account_with_address<STC>(current_author);
        };
        let current_info = RewardInfo {
            number: current_number,
            reward: current_reward,
            miner: current_author,
            gas_fees: Token::zero<STC>(),
        };
        Vector::push_back(&mut rewards.infos, current_info);

    }

    spec process_block_reward {
        aborts_if Signer::address_of(account) != CoreAddresses::GENESIS_ADDRESS();
        // abort if current block is genesis, and previous block gas fees != 0
        aborts_if current_number == 0 && Token::value(previous_block_gas_fees) != 0;

        aborts_if current_number > 0 && !exists<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
        aborts_if current_number > 0 && (global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).reward_number + Vector::length(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos) + 1) != current_number;
        aborts_if current_number > 0 && !exists<Config::Config<RewardConfig::RewardConfig>>(CoreAddresses::GENESIS_ADDRESS());


        let reward_info_length = Vector::length(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos);

        // abort if no previous block but has gas fees != 0.
        aborts_if current_number > 0 && reward_info_length == 0 && Token::value(previous_block_gas_fees) != 0;
        // abort if previous block number != current_block_number - 1.
        aborts_if current_number > 0 && reward_info_length != 0 && Vector::borrow(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos, reward_info_length - 1).number != current_number - 1;

        aborts_if current_number > 0 && Vector::length(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos) >= global<Config::Config<RewardConfig::RewardConfig>>(CoreAddresses::GENESIS_ADDRESS()).payload.reward_delay
        && (global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).reward_number + 1) != Vector::borrow(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos, 0).number;

        aborts_if current_number > 0 && Vector::length(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos) >= global<Config::Config<RewardConfig::RewardConfig>>(CoreAddresses::GENESIS_ADDRESS()).payload.reward_delay
                && (global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).reward_number + 1) > max_u64();

        aborts_if current_number > 0 && !Account::exists_at(current_author) ;

        pragma verify = false;
    }
}
}