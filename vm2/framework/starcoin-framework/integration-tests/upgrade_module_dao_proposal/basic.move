//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# faucet --addr bob

//# publish
module alice::fake_money {
    use std::option;
    use std::signer;
    use std::string;
    use std::string::{bytes, utf8};
    use starcoin_framework::coin;
    use starcoin_framework::fungible_asset::{
        generate_mint_ref,
        generate_burn_ref,
        generate_transfer_ref,
        FungibleAsset, Metadata
    };
    use starcoin_framework::primary_fungible_store::{Self, create_primary_store_enabled_fungible_asset};
    use starcoin_framework::object;
    use starcoin_framework::fungible_asset;
    use starcoin_framework::dao;

    use starcoin_std::type_info;

    struct FakeMoney {}

    struct FakeMoneyCapabilities has key {
        mint_ref: fungible_asset::MintRef,
        burn_ref: fungible_asset::BurnRef,
        transfer_ref: fungible_asset::TransferRef,
    }

    public fun init(alice: &signer, decimal: u8) {
        let struct_name = utf8(type_info::struct_name(&type_info::type_of<FakeMoney>()));
        let constructor_ref = object::create_named_object(alice, *bytes(&struct_name));
        create_primary_store_enabled_fungible_asset(
            &constructor_ref,
            option::none(), // max supply
            struct_name,
            struct_name,
            decimal,
            string::utf8(b""),
            string::utf8(b""),
        );

        let mint_ref = generate_mint_ref(&constructor_ref);
        primary_fungible_store::create_primary_store(
            signer::address_of(alice),
            fungible_asset::mint_ref_metadata(&mint_ref)
        );

        move_to(alice, FakeMoneyCapabilities {
            mint_ref,
            burn_ref: generate_burn_ref(&constructor_ref),
            transfer_ref: generate_transfer_ref(&constructor_ref),
        });

        coin::make_pair_coin_type_with_metadata<FakeMoney>(
            &constructor_ref,
            object::object_from_constructor_ref<Metadata>(&constructor_ref)
        );

        dao::plugin<FakeMoney>(
            alice,
            60 * 1000,
            60 * 60 * 1000,
            4,
            60 * 60 * 1000
        );
    }

    public fun mint(account: &signer, amount: u64): FungibleAsset acquires FakeMoneyCapabilities {
        let cap = borrow_global<FakeMoneyCapabilities>(signer::address_of(account));
        fungible_asset::mint(&cap.mint_ref, amount)
    }

    public fun burn(fa: FungibleAsset) acquires FakeMoneyCapabilities {
        let cap = borrow_global<FakeMoneyCapabilities>(@alice);
        fungible_asset::burn(&cap.burn_ref, fa);
    }

    public fun get_metadata(): object::Object<Metadata> acquires FakeMoneyCapabilities {
        let cap_holder = borrow_global<FakeMoneyCapabilities>(@alice);
        fungible_asset::mint_ref_metadata(&cap_holder.mint_ref)
    }

    public fun supply(): u64 acquires FakeMoneyCapabilities {
        (option::destroy_some(fungible_asset::supply(Self::get_metadata())) as u64)
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 1001000

//# run --signers alice
script {
    use std::option::destroy_some;
    use std::signer::address_of;

    use alice::fake_money::Self;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::fungible_asset;

    fun initialize_fake_moeney(alice: &signer) {
        fake_money::init(alice, 9);

        let meta_data = fake_money::get_metadata();
        let market_cap = destroy_some(fungible_asset::supply(meta_data));
        assert!(market_cap == 0, 8001);

        let alice_addr = address_of(alice);
        primary_fungible_store::ensure_primary_store_exists(address_of(alice), meta_data);
        assert!(primary_fungible_store::primary_store_exists(alice_addr, meta_data), 1)
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use std::signer;
    use alice::fake_money;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::fungible_asset;

    fun main(alice: &signer) {
        // mint 100 coins and check that the market cap increases appropriately
        // let coin = fake_money::mint(alice, 10000);
        // assert!(coin::value<FakeMoney>(&coin) == 10000, 8002);
        // assert!(option::destroy_some(coin::supply<FakeMoney>()) == market_cap + 10000, 8003);
        // coin::deposit<FakeMoney>(signer::address_of(&account), coin)

        let market_cap = fake_money::supply();
        let fa = fake_money::mint(alice, 10000);
        assert!(fungible_asset::amount(&fa) == 10000, 8002);
        assert!(fake_money::supply() == market_cap + 10000, 8003);
        primary_fungible_store::deposit(signer::address_of(alice), fa);
    }
}

// default upgrade strategy is arbitrary
//# run --signers alice
script {
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::signer;

    fun main(account: signer) {
        let hash = x"1111111111111111";
        stc_transaction_package_validation::check_package_txn(
            signer::address_of(&account),
            hash
        );
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
        on_chain_config::publish_new_config<stc_version::Version>(
            &account,
            stc_version::new_version(1)
        );
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

    fun test_plugin(alice: &signer) {
        let upgrade_plan_cap =
            stc_transaction_package_validation::extract_submit_upgrade_plan_cap(alice);
        dao_upgrade_module_proposal::plugin<FakeMoney>(alice, upgrade_plan_cap);
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

    fun test_propose(alice: &signer) {
        let module_address = @alice;
        let package_hash = x"1111111111111111";
        let version = 1;
        let exec_delay = 60 * 60 * 1000;
        dao_upgrade_module_proposal::propose_module_upgrade_v2<FakeMoney>(
            alice,
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
    use alice::fake_money::{Self, FakeMoney};

    use starcoin_std::debug;
    use starcoin_framework::dao_upgrade_module_proposal;
    use starcoin_framework::dao;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::signer;

    fun vote_proposal(alice: &signer) {
        let proposal_id = 0;
        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - vote_proposal | entered"));

        let state = dao::proposal_state<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(@alice, proposal_id);
        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - vote_proposal | state"));
        debug::print(&state);

        assert!(state == 2, (state as u64));
        let balance = primary_fungible_store::balance(
            signer::address_of(alice),
            fake_money::get_metadata(),
        );

        debug::print(&string::utf8(b"upgrade_module_dao_proposal/basic.move - vote_proposal | account balance"));
        debug::print(&balance);

        // let balance = coin::withdraw<FakeMoney>(&account, balance / 2);
        let fa = primary_fungible_store::withdraw(alice, fake_money::get_metadata(), balance / 2);
        dao::cast_vote<FakeMoney, dao_upgrade_module_proposal::UpgradeModuleV2>(
            alice,
            @alice,
            proposal_id,
            fa,
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

