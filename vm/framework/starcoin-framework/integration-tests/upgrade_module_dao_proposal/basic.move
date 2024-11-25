//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# faucet --addr bob

//# publish
module alice::fake_money {
    use std::signer;
    use std::string;

    use starcoin_framework::coin;
    use starcoin_framework::dao;

    struct FakeMoney {}

    struct FakeMoneyCapabilities has key {
        burn_cap: coin::BurnCapability<FakeMoney>,
        freeze_cap: coin::FreezeCapability<FakeMoney>,
        mint_cap: coin::MintCapability<FakeMoney>,
    }

    public fun init(account: &signer, decimal: u8) {
        let (
            burn_cap,
            freeze_cap,
            mint_cap
        ) = coin::initialize<FakeMoney>(
            account,
            string::utf8(b"FakeMoney"),
            string::utf8(b"FakeMoney"),
            decimal,
            true,
        );
        coin::register<FakeMoney>(account);
        dao::plugin<FakeMoney>(account, 60 * 1000, 60 * 60 * 1000, 4, 60 * 60 * 1000);
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        })
    }

    public fun mint(account: &signer, amount: u64): coin::Coin<FakeMoney> acquires FakeMoneyCapabilities {
        let cap = borrow_global<FakeMoneyCapabilities>(signer::address_of(account));
        coin::mint(amount, &cap.mint_cap)
    }

    public fun burn(coin: coin::Coin<FakeMoney>) acquires FakeMoneyCapabilities {
        let cap = borrow_global<FakeMoneyCapabilities>(@alice);
        coin::burn(coin, &cap.burn_cap)
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 1001000

//# run --signers alice
script {
    use std::option;
    use std::string;
    use alice::fake_money::{Self, FakeMoney};
    use starcoin_std::debug;
    use starcoin_framework::coin;

    fun initialize_fake_moeney(account: signer) {
        fake_money::init(&account, 9);

        let market_cap = option::destroy_some(coin::supply<FakeMoney>());
        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - initialize_fake_money | market_cap: "));
        debug::print(&market_cap);

        assert!(market_cap == 0, 8001);
        assert!(coin::is_account_registered<FakeMoney>(@alice), 8002);
        // Create 'Balance<TokenType>' resource under sender account, and init with zero
        // account::do_accept_token<FakeMoney>(&account);
        coin::register<FakeMoney>(&account);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use std::option;
    use std::signer;
    use alice::fake_money::{Self, FakeMoney};
    use starcoin_framework::coin;

    fun main(account: signer) {
        // mint 100 coins and check that the market cap increases appropriately
        let market_cap = option::destroy_some(coin::supply<FakeMoney>());
        let coin = fake_money::mint(&account, 10000);
        assert!(coin::value<FakeMoney>(&coin) == 10000, 8002);
        assert!(option::destroy_some(coin::supply<FakeMoney>()) == market_cap + 10000, 8003);
        coin::deposit<FakeMoney>(signer::address_of(&account), coin)
    }
}

// default upgrade strategy is arbitrary
//# run --signers alice
script {
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::signer;

    fun main(account: signer) {
        let hash = x"1111111111111111";
        stc_transaction_package_validation::check_package_txn(signer::address_of(&account), hash);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::on_chain_config;
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::stc_version::Self;
    use std::option;

    fun main(account: signer) {
        on_chain_config::publish_new_config<stc_version::Version>(&account, stc_version::new_version(1));
        stc_transaction_package_validation::update_module_upgrade_strategy(
            &account,
            stc_transaction_package_validation::get_strategy_two_phase(),
            option::some<u64>(0)
        );
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::starcoin_coin::STC;

    fun test_plugin_fail(account: signer) {
        let upgrade_plan_cap =
            stc_transaction_package_validation::extract_submit_upgrade_plan_cap(&account);
        dao_upgrade_module_proposal::plugin<STC>(&account, upgrade_plan_cap); //ERR_NOT_AUTHORIZED
    }
}
// check: ERR_NOT_AUTHORIZED


//# run --signers alice
script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::stc_transaction_package_validation;
    use alice::fake_money::FakeMoney;


    fun test_plugin(account: signer) {
        let upgrade_plan_cap =
            stc_transaction_package_validation::extract_submit_upgrade_plan_cap(&account);
        dao_upgrade_module_proposal::plugin<FakeMoney>(&account, upgrade_plan_cap);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use alice::fake_money::FakeMoney;
    use starcoin_framework::dao_upgrade_module_proposal;

    fun test_propose_fail(account: signer) {
        let module_address = @alice;
        let package_hash = x"1111111111111111";
        let version = 1;
        let exec_delay = 1;
        dao_upgrade_module_proposal::propose_module_upgrade_v2<FakeMoney>(
            &account,
            module_address, // ERR_ADDRESS_MISSMATCH
            copy package_hash,
            version,
            exec_delay,
            false,
        );
    }
}
// check: FAILED

//# run --signers alice
script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use alice::fake_money::FakeMoney;

    fun test_propose(account: signer) {
        let module_address = @alice;
        let package_hash = x"1111111111111111";
        let version = 1;
        let exec_delay = 60 * 60 * 1000;
        dao_upgrade_module_proposal::propose_module_upgrade_v2<FakeMoney>(
            &account,
            module_address,
            copy package_hash,
            version,
            exec_delay,
            false,
        );
    }
}


//# block --author 0x1 --timestamp 4601000

//# run --signers alice
script {
    use std::string;
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::dao;
    use alice::fake_money::FakeMoney;
    use starcoin_std::debug;
    use starcoin_framework::coin;
    use starcoin_framework::signer;

    fun vote_proposal(account: signer) {
        let proposal_id = 0;
        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - vote_proposal | entered"));

        let state = dao::proposal_state<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(@alice, proposal_id);
        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - vote_proposal | state"));
        debug::print(&state);

        assert!(state == 2, (state as u64));
        let balance = coin::balance<FakeMoney>(signer::address_of(&account));

        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - vote_proposal | account balance"));
        debug::print(&balance);

        let balance = coin::withdraw<FakeMoney>(&account, balance / 2);
        dao::cast_vote<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(
            &account,
            @alice,
            proposal_id,
            balance,
            true
        );

        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - vote_proposal | exited"));
    }
}


//# block --author 0x1 --timestamp 8262000

//# run --signers alice
script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::dao;
    use alice::fake_money::FakeMoney;

    fun queue_proposal(_signer: signer) {
        let proposal_id = 0;
        let state = dao::proposal_state<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(@alice, proposal_id);
        assert!(state == 4, (state as u64));
        dao::queue_proposal_action<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(@alice, proposal_id);
        let state = dao::proposal_state<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(@alice, proposal_id);
        assert!(state == 5, (state as u64));
    }
}

//# block --author 0x1 --timestamp 15262000

//# run --signers alice
script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use alice::fake_money::FakeMoney;
    use starcoin_framework::dao;

    fun test_submit_plan(_account: signer) {
        let proposal_id = 0;
        let proposer_address = @alice;
        let state = dao::proposal_state<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(
            proposer_address,
            proposal_id
        );
        assert!(state == 6, (state as u64));
        dao_upgrade_module_proposal::submit_module_upgrade_plan<FakeMoney>(proposer_address, proposal_id);
    }
}

