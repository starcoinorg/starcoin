address 0x1 {

module BlockReward {
    use 0x1::Timestamp;
    use 0x1::Token;
    use 0x1::STC::{STC};
    use 0x1::Vector;
    use 0x1::Account;
    use 0x1::CoreAddresses;
    use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::RewardConfig;
    use 0x1::Config;
    use 0x1::Authenticator;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    resource struct RewardQueue {
        reward_number: u64,
        infos: vector<RewardInfo>,
    }

    struct RewardInfo {
        number: u64,
        reward: u128,
        miner: address,
    }

    const EAUTHOR_PUBLIC_KEY_IS_NOT_EMPTY: u64 = 101;
    const ECURRENT_NUMBER_IS_WRONG: u64 = 102;
    const EREWARD_NUMBER_IS_WRONG: u64 = 103;
    const EMINER_EXIST: u64 = 104;

    public fun initialize(account: &signer, reward_delay: u64) {
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);

        RewardConfig::initialize(account, reward_delay);
        move_to<RewardQueue>(account, RewardQueue {
            reward_number: 0,
            infos: Vector::empty(),
        });
    }

    spec fun initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::GENESIS_ADDRESS();
        include Config::PublishNewConfigAbortsIf<RewardConfig::RewardConfig>;
        include Config::PublishNewConfigEnsures<RewardConfig::RewardConfig>;
        aborts_if exists<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
        ensures exists<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
    }

    public fun process_block_reward(account: &signer, current_number: u64, current_reward: u128,
                                    current_author: address, public_key_vec: vector<u8>) acquires RewardQueue {
        CoreAddresses::assert_genesis_address(account);

        if (current_number > 0) {
            let rewards = borrow_global_mut<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
            let len = Vector::length(&rewards.infos);
            assert((current_number == (rewards.reward_number + len + 1)), Errors::invalid_argument(ECURRENT_NUMBER_IS_WRONG));

            let reward_delay = RewardConfig::reward_delay();
            if (len >= reward_delay) {//pay and remove
                let i = len;
                while (i > 0 && i >= reward_delay) {
                    let reward_number = rewards.reward_number + 1;
                    let first_info = *Vector::borrow(&rewards.infos, 0);
                    assert((reward_number == first_info.number), Errors::invalid_argument(EREWARD_NUMBER_IS_WRONG));

                    rewards.reward_number = reward_number;
                    if (first_info.reward > 0) {
                        assert(Account::exists_at(first_info.miner), Errors::requires_address(EMINER_EXIST));
                        let reward = Token::mint<STC>(account, first_info.reward);
                        Account::deposit_to<STC>(account, first_info.miner, reward);
                    };
                    Vector::remove(&mut rewards.infos, 0);
                    i = i - 1;
                }
            };

            if (!Account::exists_at(current_author)) {
                //create account from public key
                assert(!Vector::is_empty(&public_key_vec), Errors::invalid_argument(EAUTHOR_PUBLIC_KEY_IS_NOT_EMPTY));
                Account::create_account<STC>(current_author, public_key_vec);
            };
            let current_info = RewardInfo {
                number: current_number,
                reward: current_reward,
                miner: current_author,
            };
            Vector::push_back(&mut rewards.infos, current_info);
        };
    }

    spec fun process_block_reward {
        aborts_if Signer::address_of(account) != CoreAddresses::GENESIS_ADDRESS();
        aborts_if current_number > 0 && !exists<RewardQueue>(CoreAddresses::GENESIS_ADDRESS());
        aborts_if current_number > 0 && (global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).reward_number + Vector::length(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos) + 1) != current_number;
        aborts_if current_number > 0 && !exists<Config::Config<RewardConfig::RewardConfig>>(CoreAddresses::GENESIS_ADDRESS());

        aborts_if current_number > 0 && Vector::length(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos) >= global<Config::Config<RewardConfig::RewardConfig>>(CoreAddresses::GENESIS_ADDRESS()).payload.reward_delay
        && (global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).reward_number + 1) != Vector::borrow(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos, 0).number;

        aborts_if current_number > 0 && Vector::length(global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).infos) >= global<Config::Config<RewardConfig::RewardConfig>>(CoreAddresses::GENESIS_ADDRESS()).payload.reward_delay
                && (global<RewardQueue>(CoreAddresses::GENESIS_ADDRESS()).reward_number + 1) > max_u64();

        aborts_if current_number > 0 && !Account::exists_at(current_author) && Vector::is_empty(public_key_vec);
        aborts_if current_number > 0 && !Account::exists_at(current_author) && len(Authenticator::spec_ed25519_authentication_key(public_key_vec)) != 32;
        aborts_if current_number > 0 && !Account::exists_at(current_author) && Authenticator::spec_derived_address(Authenticator::spec_ed25519_authentication_key(public_key_vec)) != current_author;

        pragma verify = false;
    }
}
}