//! account: admin
//! account: staker
//! account: staker2

//! new-transaction
//! sender: admin
module Token1 {
    struct Token1 { }
}
// check: EXECUTED

//! block-prologue
//! author: admin
//! block-number: 1
//! block-time: 100

//! new-transaction
//! sender: admin
script {
    use {{genesis}}::RewardPool;
    use {{admin}}::Token1;
    use 0x1::STC;
    use 0x1::Token;
    fun create_reward_pool(signer: &signer) {
        Token::register_token<Token1::Token1>(signer, 1000000, 1000);
        let reward_token = Token::mint<Token1::Token1>(signer, 100000000);
        // 1000 blocks.
        let duration = 5;
        // create a reward pool
        RewardPool::initialize<STC::STC, Token1::Token1>(signer, reward_token, duration);
    }
}

// check: EXECUTED

//! block-prologue
//! author: admin
//! block-number: 2
//! block-time: 101


//! new-transaction
//! sender: staker
script {
    use {{genesis}}::RewardPool;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Option;
    fun stake(signer: &signer) {
        Account::accept_token<Token1::Token1>(signer);
        let my_stc = Account::withdraw<STC::STC>(signer, 10000);
        RewardPool::stake<STC::STC, Token1::Token1>(signer, {{admin}}, my_stc);
        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker}}, {{admin}});
        assert(earned == 0, 100);
        let staked = RewardPool::staked_share<STC::STC, Token1::Token1>({{staker}}, {{admin}});
        assert(Option::is_some(&staked), 100);
        let staked = Option::extract(&mut staked);
        assert(staked == 10000, (staked as u64));
    }
}
// check: EXECUTED

//! block-prologue
//! author: admin
//! block-number: 3
//! block-time: 102


//! new-transaction
//! sender: staker
script {
    use {{genesis}}::RewardPool;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Option;
    fun unstake(signer: &signer) {
        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker}}, {{admin}});
        let reward_rate = (100000000 / 5);
        let expected_earned: u128 = reward_rate * (3 - 2);
        assert(earned == expected_earned, (earned as u64));
        let my_stc = RewardPool::unstake<STC::STC, Token1::Token1>(signer, {{admin}}, 5000);
        Account::deposit(signer, my_stc);

        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker}}, {{admin}});
        assert(earned == expected_earned, (earned as u64));
        let staked = RewardPool::staked_share<STC::STC, Token1::Token1>({{staker}}, {{admin}});
        assert(Option::is_some(&staked), 100);
        let staked = Option::extract(&mut staked);
        assert(staked == 5000, (staked as u64));
    }
}
// check: EXECUTED

//! new-transaction
//! sender: staker2
script {
    use {{genesis}}::RewardPool;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Option;
    fun stake(signer: &signer) {
        Account::accept_token<Token1::Token1>(signer);
        let my_stc = Account::withdraw<STC::STC>(signer, 5000);
        RewardPool::stake<STC::STC, Token1::Token1>(signer, {{admin}}, my_stc);
        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker2}}, {{admin}});
        assert(earned == 0, 100);
        let staked = RewardPool::staked_share<STC::STC, Token1::Token1>({{staker2}}, {{admin}});
        assert(Option::is_some(&staked), 100);
        let staked = Option::extract(&mut staked);
        assert(staked == 5000, (staked as u64));
        let total_staked = RewardPool::total_staked_shares<STC::STC, Token1::Token1>({{admin}});
        assert(total_staked == 10000, 10000);
    }
}
// check: EXECUTED



//! block-prologue
//! author: admin
//! block-number: 4
//! block-time: 103


//! block-prologue
//! author: admin
//! block-number: 5
//! block-time: 104

//! new-transaction
//! sender: staker
script {
    use {{genesis}}::RewardPool;
    use 0x1::STC;
    use {{admin}}::Token1;
    fun check_earned(_signer: &signer) {
        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker}}, {{admin}});
        let reward_rate = (100000000 / 5);
        let expected_earned: u128 = reward_rate * (3 - 2) + reward_rate / 2 * (5 - 3);
        assert(earned == expected_earned, (earned as u64));
    }
}
// check: EXECUTED


//! new-transaction
//! sender: staker2
script {
    use {{genesis}}::RewardPool;
    use 0x1::STC;
    use {{admin}}::Token1;
    fun check_earned(_signer: &signer) {
        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker2}}, {{admin}});
        let reward_rate = (100000000 / 5);
        let expected_earned: u128 = reward_rate / 2 * (5 - 3);
        assert(earned == expected_earned, (earned as u64));
    }
}
// check: EXECUTED


//! block-prologue
//! author: admin
//! block-number: 6
//! block-time: 105


//! new-transaction
//! sender: staker
script {
    use {{genesis}}::RewardPool;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Token;
    fun exit(signer: &signer) {
        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker}}, {{admin}});
        let reward_rate = (100000000 / 5);
        let expected_earned: u128 = reward_rate * (3 - 2) + reward_rate / 2 * (6 - 3);
        assert(earned == expected_earned, (earned as u64));
        let rewards = RewardPool::withdraw_rewards<STC::STC, Token1::Token1>(signer, {{admin}});
        let actual_reward = Token::share(&rewards);
        assert(actual_reward == expected_earned, (actual_reward as u64));
        Account::deposit(signer, rewards);
        let (stake_token, reward_token) = RewardPool::exit<STC::STC, Token1::Token1>(signer, {{admin}});
        {
            let staked= Token::share(&stake_token);
            assert(staked == 5000, (staked as u64));
        };

        {
            let reward = Token::share(&reward_token);
            assert(reward == 0, (reward as u64));
        };
        Token::destroy_zero(reward_token);
        Account::deposit(signer, stake_token);
    }
}

// check: EXECUTED

//! block-prologue
//! author: admin
//! block-number: 7
//! block-time: 108

//! new-transaction
//! sender: staker2
script {
    use {{genesis}}::RewardPool;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Token;
    fun exit(signer: &signer) {
        let earned = RewardPool::earned<STC::STC, Token1::Token1>({{staker2}}, {{admin}});
        let reward_rate = (100000000 / 5);
        let expected_earned: u128 = reward_rate / 2 * (6 - 3);
        assert(earned == expected_earned, (earned as u64));
        let rewards = RewardPool::withdraw_rewards<STC::STC, Token1::Token1>(signer, {{admin}});
        let actual_reward = Token::share(&rewards);
        assert(actual_reward == expected_earned, (actual_reward as u64));
        Account::deposit(signer, rewards);
        let (stake_token, reward_token) = RewardPool::exit<STC::STC, Token1::Token1>(signer, {{admin}});
        {
            let staked= Token::share(&stake_token);
            assert(staked == 5000, (staked as u64));
        };

        {
            let reward = Token::share(&reward_token);
            assert(reward == 0, (reward as u64));
        };
        Token::destroy_zero(reward_token);
        Account::deposit(signer, stake_token);
    }
}
// check: EXECUTED
