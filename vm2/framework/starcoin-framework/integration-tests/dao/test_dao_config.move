//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# publish
module alice::my_coin {
    use std::option;
    use std::signer;
    use std::string::{Self, bytes, utf8};

    use starcoin_framework::dao;
    use starcoin_framework::fungible_asset::{Self, generate_burn_ref, generate_mint_ref, generate_transfer_ref,
        Metadata
    };
    use starcoin_framework::object;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::primary_fungible_store::create_primary_store_enabled_fungible_asset;
    use starcoin_std::type_info;

    struct MyCoin has copy, drop, store {}

    struct CapabilityHolder<phantom TokenT> has key {
        mint_ref: fungible_asset::MintRef,
        burn_ref: fungible_asset::BurnRef,
        transfer_ref: fungible_asset::TransferRef,
    }

    public fun init(alice: &signer) {
        let struct_name = utf8(type_info::struct_name(&type_info::type_of<MyCoin>()));
        let constructor_ref = object::create_named_object(alice, *bytes(&struct_name));
        create_primary_store_enabled_fungible_asset(
            &constructor_ref,
            option::some(100), // max supply
            struct_name,
            struct_name,
            9,
            string::utf8(b""),
            string::utf8(b""),
        );

        let mint_ref = generate_mint_ref(&constructor_ref);
        primary_fungible_store::create_primary_store(signer::address_of(alice), fungible_asset::mint_ref_metadata(&mint_ref));

        move_to(alice, CapabilityHolder<MyCoin> {
            mint_ref,
            burn_ref: generate_burn_ref(&constructor_ref),
            transfer_ref: generate_transfer_ref(&constructor_ref),
        });

        dao::plugin<MyCoin>(
            alice,
            60 * 1000,
            60 * 60 * 1000,
            4,
            60 * 60 * 1000
        );
    }

    public fun get_metadata(): object::Object<Metadata> acquires CapabilityHolder {
        let cap_holder = borrow_global<CapabilityHolder<MyCoin>>(@alice);
        fungible_asset::mint_ref_metadata(&cap_holder.mint_ref)
    }
}

//# run --signers alice
script {
    use alice::my_coin;
    use std::option;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::fungible_asset;

    fun main(alice: &signer) {
        my_coin::init(alice);
        let meta_data = my_coin::get_metadata();
        let market_cap = fungible_asset::supply(meta_data);
        assert!(option::destroy_some(market_cap) == 0, 8001);
        assert!(primary_fungible_store::primary_store_exists(@alice, meta_data), 8002);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;
    use starcoin_framework::on_chain_config;


    fun set_dao_config(signer: signer) {
        let cap = on_chain_config::extract_modify_config_capability<dao::DaoConfig<MyCoin>>(
            &signer
        );

        dao::set_voting_delay<MyCoin>(&mut cap, 30 * 1000);
        dao::set_voting_period<MyCoin>(&mut cap, 30 * 30 * 1000);
        dao::set_voting_quorum_rate<MyCoin>(&mut cap, 50);
        dao::set_min_action_delay<MyCoin>(&mut cap, 30 * 30 * 1000);

        on_chain_config::restore_modify_config_capability(cap);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;
    use starcoin_framework::on_chain_config;

    fun set_dao_config(signer: signer) {
        let cap =
            on_chain_config::extract_modify_config_capability<dao::DaoConfig<MyCoin>>(
            &signer
            );
        dao::set_voting_delay<MyCoin>(&mut cap, 0);
        on_chain_config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code 66943"


//# run --signers alice

script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;
    use starcoin_framework::on_chain_config;

    fun set_dao_config(signer: signer) {
        let cap =
            on_chain_config::extract_modify_config_capability<dao::DaoConfig<MyCoin>>(
            &signer
            );
        dao::set_voting_period<MyCoin>(&mut cap, 0);
        on_chain_config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;
    use starcoin_framework::on_chain_config;

    fun set_dao_config(signer: signer) {
        let cap = on_chain_config::extract_modify_config_capability<dao::DaoConfig<MyCoin>>(
            &signer
        );
        dao::set_voting_quorum_rate<MyCoin>(&mut cap, 0);
        on_chain_config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 359943"


//# run --signers alice
script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;
    use starcoin_framework::on_chain_config;

    fun set_dao_config(signer: signer) {
        let cap =
            on_chain_config::extract_modify_config_capability<dao::DaoConfig<MyCoin>>(
            &signer
            );
        dao::set_min_action_delay<MyCoin>(&mut cap, 0);
        on_chain_config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers bob
script {
    use alice::my_coin::MyCoin;
    use starcoin_framework::dao_modify_config_proposal;

    fun test_plugin(signer: signer) {
        dao_modify_config_proposal::plugin<MyCoin>(&signer); //ERR_NOT_AUTHORIZED
    }
}
// check: "Keep(ABORTED { code: 102658"


//# run --signers alice
script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;
    use starcoin_framework::on_chain_config;

    fun modify_dao_config(signer: signer) {
        let cap = on_chain_config::extract_modify_config_capability<dao::DaoConfig<MyCoin>>(
            &signer
        );
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 50;
        let min_action_delay = 30 * 30 * 1000;

        dao::modify_dao_config<MyCoin>(
            &mut cap,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );

        let voting_delay = 0;
        let voting_period = 0;
        let voting_quorum_rate = 0;
        let min_action_delay = 0;

        dao::modify_dao_config<MyCoin>(
            &mut cap,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );

        on_chain_config::restore_modify_config_capability(cap);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;
    use starcoin_framework::on_chain_config;

    fun modify_dao_config(signer: signer) {
        let cap = on_chain_config::extract_modify_config_capability<dao::DaoConfig<MyCoin>>(
            &signer
        );
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 101; //ERR_QUORUM_RATE_INVALID
        let min_action_delay = 30 * 30 * 1000;

        dao::modify_dao_config<MyCoin>(
            &mut cap,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );

        on_chain_config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 359943"


//# run --signers alice

script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 0; //should > 0
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 50;
        let min_action_delay = 30 * 30 * 1000;

        dao::new_dao_config<MyCoin>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 0; //should > 0
        let voting_quorum_rate = 50;
        let min_action_delay = 30 * 30 * 1000;

        dao::new_dao_config<MyCoin>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice
script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 0; //should > 0
        let min_action_delay = 30 * 30 * 1000;

        dao::new_dao_config<MyCoin>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 101; //should <= 100
        let min_action_delay = 30 * 30 * 1000;

        dao::new_dao_config<MyCoin>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use starcoin_framework::dao;
    use alice::my_coin::MyCoin;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 50;
        let min_action_delay = 0; //should > 0

        dao::new_dao_config<MyCoin>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"