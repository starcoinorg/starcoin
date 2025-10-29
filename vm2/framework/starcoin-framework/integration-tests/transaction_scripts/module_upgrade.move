//# init -n dev


//# faucet --addr alice

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


//# block --author 0x1 --timestamp 2601000

//# run --signers alice
script {
    use std::signer;
    use alice::fake_money::Self;
    use starcoin_framework::primary_fungible_store;

    fun main(alice: signer) {
        fake_money::init(&alice, 9);

        let fa = fake_money::mint(&alice, 10000);
        primary_fungible_store::deposit(signer::address_of(&alice), fa);
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
    use starcoin_framework::dao;
    use starcoin_framework::dao_upgrade_module_proposal;

    use alice::fake_money::{Self, FakeMoney};
    use starcoin_framework::primary_fungible_store;

    fun cast_vote(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let stake = primary_fungible_store::withdraw(
            &signer,
            fake_money::get_metadata(),
            (votes as u64)
        );
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
