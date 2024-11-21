//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# block --author 0x1 --timestamp 86400000


//# run --signers StarcoinAssociation
script {
    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;

    fun transfer_some_token_to_alice_and_bob(signer: signer) {
        let balance = coin::balance<STC>(signer::address_of(&signer));
        coin::transfer<STC>(&signer, @alice, balance / 4);
        coin::transfer<STC>(&signer, @bob, balance / 4);
    }
}
//# run --signers alice
script {
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::starcoin_coin::STC;

    fun test_plugin_fail(account: signer) {
        on_chain_config_dao::plugin<STC, on_chain_config_dao::OnChainConfigUpdate<transaction_publish_option::TransactionPublishOption>>(
            &account
        ); //ERR_NOT_AUTHORIZED
    }
}
//# run --signers alice
script {
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(signer: signer) {
        let new_config = transaction_publish_option::new_transaction_publish_option(true, false);
        on_chain_config_dao::propose_update<STC, transaction_publish_option::TransactionPublishOption>(
            &signer,
            new_config,
            0
        );
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 87000000

//# run --signers bob
script {
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::coin;
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;
    use starcoin_framework::dao;

    fun vote(signer: signer) {
        let balance = coin::balance<STC>(signer::address_of(&signer));
        let balance = coin::withdraw<STC>(&signer, balance / 2);
        dao::cast_vote<STC, on_chain_config_dao::OnChainConfigUpdate<transaction_publish_option::TransactionPublishOption>>(
            &signer,
            @alice,
            0,
            balance,
            true
        );
    }
}


//# block --author 0x1 --timestamp 110000000

//# block --author 0x1 --timestamp 120000000

//# run --signers bob
script {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun queue_proposal(signer: signer) {
        let state = dao::proposal_state<STC, on_chain_config_dao::OnChainConfigUpdate<transaction_publish_option::TransactionPublishOption>>(
            @alice,
            0
        );
        assert!(state == 4, (state as u64));
        {
            let token = dao::unstake_votes<STC, on_chain_config_dao::OnChainConfigUpdate<transaction_publish_option::TransactionPublishOption>>(
                &signer,
                @alice,
                0
            );
            coin::deposit(signer::address_of(&signer), token);
        };
        dao::queue_proposal_action<STC, on_chain_config_dao::OnChainConfigUpdate<transaction_publish_option::TransactionPublishOption>>(
            @alice,
            0
        );
        // ModifyDaoConfigProposal::execute<STC>(@alice, 0);
        let state = dao::proposal_state<STC, on_chain_config_dao::OnChainConfigUpdate<transaction_publish_option::TransactionPublishOption>>(
            @alice,
            0
        );
        assert!(state == 5, (state as u64));
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 130000000

//# run --signers bob
script {
    use starcoin_framework::transaction_publish_option;
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun execute_proposal_action(_signer: signer) {
        let state = dao::proposal_state<STC, on_chain_config_dao::OnChainConfigUpdate<transaction_publish_option::TransactionPublishOption>>(
            @alice,
            0
        );
        assert!(state == 6, (state as u64));
        on_chain_config_dao::execute<STC, transaction_publish_option::TransactionPublishOption>(@alice, 0);
        assert!(!transaction_publish_option::is_module_allowed(@starcoin_framework), 401);
        assert!(transaction_publish_option::is_script_allowed(@starcoin_framework), 402);
    }
}
// check: EXECUTED

