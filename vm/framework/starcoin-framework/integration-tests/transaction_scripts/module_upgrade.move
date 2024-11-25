//# init -n dev


//# faucet --addr alice

//# faucet --addr bob


//# publish
module alice::MyToken {
    use starcoin_framework::Token;
    use starcoin_framework::dao;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer) {
        Token::register_token<MyToken>(
            account,
            3,
        );
        dao::plugin<MyToken>(account, 60 * 1000, 60 * 60 * 1000, 4, 60 * 60 * 1000);
    }
}

//# block --author 0x1 --timestamp 2601000

//# run --signers alice
script {
    use alice::MyToken::{MyToken, Self};
    use starcoin_framework::account;
    use starcoin_framework::Token;

    fun main(account: signer) {
        MyToken::init(&account);
        account::do_accept_token<MyToken>(&account);
        let coin = Token::mint<MyToken>(&account, 10000);
        coin::deposit<MyToken>(&account, coin)
    }
}

//# run --signers alice
script {
    use starcoin_framework::on_chain_config;
    use starcoin_framework::stc_version::Version;
    use starcoin_framework::PackageTxnManager;
    use std::option;

    fun update_module_upgrade_strategy(account: signer) {
        Config::publish_new_config<Version::Version>(&account, Version::new_version(1));
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_two_phase(), option::some<u64>(1));
    }
}

//# run --signers alice
script {
    use starcoin_framework::UpgradeModuleDaoProposal;
    use starcoin_framework::PackageTxnManager;
    use alice::MyToken::MyToken;

    fun plugin(account: signer) {
        let upgrade_plan_cap = PackageTxnManager::extract_submit_upgrade_plan_cap(&account);
        UpgradeModuleDaoProposal::plugin<MyToken>(&account, upgrade_plan_cap);
    }
}


//# run --signers alice
script {
    use starcoin_framework::UpgradeModuleDaoProposal;
    use alice::MyToken::MyToken;

    fun propose_module_upgrade(account: signer) {
        let module_address = @alice;
        let package_hash = x"1111111111111111";
        let version = 1;
        let exec_delay = 60 * 60 * 1000;
        UpgradeModuleDaoProposal::propose_module_upgrade_v2<MyToken>(
            &account,
            module_address,
            copy package_hash,
            version,
            exec_delay,
            false,
        );
    }
}



//# block --author 0x1 --timestamp 3601000

//# run --signers alice --args @alice --args 0 --args true --args 500u128
script {
    use starcoin_framework::UpgradeModuleDaoProposal;
    use starcoin_framework::dao;
    use starcoin_framework::account;
    use alice::MyToken::MyToken;

    fun cast_vote(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let stake = coin::withdraw<MyToken>(&signer, votes);
        dao::cast_vote<MyToken, UpgradeModuleDaoProposal::UpgradeModuleV2>(&signer, proposer_address, proposal_id, stake, agree);
    }
}

//# block --author 0x1 --timestamp 7662000

//# run --signers alice --args @alice --args 0

script {
    use starcoin_framework::UpgradeModuleDaoProposal;
    use starcoin_framework::dao;
    use alice::MyToken::MyToken;

    fun queue_proposal_action(_signer: signer,
        proposer_address: address,
        proposal_id: u64
    ) {
        dao::queue_proposal_action<MyToken, UpgradeModuleDaoProposal::UpgradeModuleV2>(proposer_address, proposal_id);
    }
}

//# block --author 0x1 --timestamp 12262000


//# run --signers alice --args @alice --args 0

script {
    use starcoin_framework::UpgradeModuleDaoProposal;
    use alice::MyToken::MyToken;

    fun submit_module_upgrade_plan(_account: signer, proposer_address: address, proposal_id: u64) {
        UpgradeModuleDaoProposal::submit_module_upgrade_plan<MyToken>(proposer_address, proposal_id);
    }
}
