//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# publish
module alice::my_coin {
    use std::string;

    use starcoin_framework::coin;
    use starcoin_framework::coin::{BurnCapability, FreezeCapability, MintCapability};
    use starcoin_framework::dao;

    struct MyCoin has copy, drop, store { }

    struct CapabilityHolder has key {
        burn_cap: BurnCapability<MyCoin>,
        freeze_cap: FreezeCapability<MyCoin>,
        mint_cap: MintCapability<MyCoin>,
    }

    public fun init(account: &signer) {
        let (
            burn_cap,
            freeze_cap,
            mint_cap
        ) = coin::initialize<MyCoin>(
            account,
            string::utf8(b"MyCoin"),
            string::utf8(b"MyCoin"),
            8,
            true
        );
        coin::register<MyCoin>(account);
        dao::plugin<MyCoin>(
            account,
            60 * 1000,
            60 * 60 * 1000,
            4,
            60 * 60 * 1000
        );
        move_to(account, CapabilityHolder {
            burn_cap,
            freeze_cap,
            mint_cap,
        });
    }
}


//# run --signers alice
script {
    use alice::my_coin::{Self, MyCoin};
    use std::option;
    use starcoin_framework::coin;

    fun main(account: signer) {
        my_coin::init(&account);
        let market_cap = coin::supply<MyCoin>();
        assert!(option::destroy_some(market_cap) == 0, 8001);
        assert!(coin::is_account_registered<MyCoin>(@alice), 8002);
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