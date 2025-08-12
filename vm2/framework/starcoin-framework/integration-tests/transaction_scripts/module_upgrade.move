//# init -n dev


//# faucet --addr alice

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


//# block --author 0x1 --timestamp 2601000

//# run --signers alice
script {
    use std::signer;
    use alice::fake_money::{Self, FakeMoney};
    use starcoin_framework::coin;

    fun main(account: signer) {
        fake_money::init(&account, 9);
        coin::register<FakeMoney>(&account);
        let coin = fake_money::mint(&account, 10000);
        coin::deposit<FakeMoney>(signer::address_of(&account), coin)
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::on_chain_config;
    use starcoin_framework::stc_version::{Self, Version};
    use starcoin_framework::stc_transaction_package_validation;
    use std::option;

    fun update_module_upgrade_strategy(account: signer) {
        on_chain_config::publish_new_config<Version>(&account, stc_version::new_version(1));
        stc_transaction_package_validation::update_module_upgrade_strategy(
            &account,
            stc_transaction_package_validation::get_strategy_two_phase(),
            option::some<u64>(1)
        );
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use alice::fake_money::FakeMoney;
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::stc_transaction_package_validation;

    fun plugin(account: signer) {
        let upgrade_plan_cap = stc_transaction_package_validation::extract_submit_upgrade_plan_cap(&account);
        dao_upgrade_module_proposal::plugin<FakeMoney>(&account, upgrade_plan_cap);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use alice::fake_money::FakeMoney;

    fun propose_module_upgrade(account: signer) {
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
// check: EXECUTED


//# block --author 0x1 --timestamp 3601000

//# run --signers alice --args @alice --args 0 --args true --args 500u128
script {
    use starcoin_framework::coin;
    use starcoin_framework::dao;
    use starcoin_framework::dao_upgrade_module_proposal;

    use alice::fake_money::FakeMoney;

    fun cast_vote(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let stake = coin::withdraw<FakeMoney>(&signer, (votes as u64));
        dao::cast_vote<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(
            &signer,
            proposer_address,
            proposal_id,
            stake,
            agree
        );
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 7662000

//# run --signers alice --args @alice --args 0

script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::dao;
    use alice::fake_money::FakeMoney;

    fun queue_proposal_action(
        _signer: signer,
        proposer_address: address,
        proposal_id: u64
    ) {
        dao::queue_proposal_action<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(
            proposer_address,
            proposal_id
        );
    }
}

//# block --author 0x1 --timestamp 12262000


//# run --signers alice --args @alice --args 0

script {
    use starcoin_framework::dao_upgrade_module_proposal;
    use alice::fake_money::FakeMoney;

    fun submit_module_upgrade_plan(_account: signer, proposer_address: address, proposal_id: u64) {
        dao_upgrade_module_proposal::submit_module_upgrade_plan<FakeMoney>(proposer_address, proposal_id);
    }
}
