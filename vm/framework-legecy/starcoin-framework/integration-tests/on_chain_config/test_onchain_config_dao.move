//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# block --author 0x1 --timestamp 86400000


//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Signer;

    fun transfer_some_token_to_alice_and_bob(signer: signer) {
        let balance = Account::balance<STC>(Signer::address_of(&signer));
        Account::pay_from<STC>(&signer, @alice, balance / 4);
        Account::pay_from<STC>(&signer, @bob, balance / 4);
    }
}
//# run --signers alice
script {
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::OnChainConfigDao;
    use StarcoinFramework::TransactionPublishOption;

    fun test_plugin_fail(account: signer) {
        OnChainConfigDao::plugin<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(&account); //ERR_NOT_AUTHORIZED
    }
}
//# run --signers alice
script {
    use StarcoinFramework::OnChainConfigDao;
    use StarcoinFramework::TransactionPublishOption;
    use StarcoinFramework::STC::STC;
    fun propose(signer: signer) {
        let new_config = TransactionPublishOption::new_transaction_publish_option(true, false);
        OnChainConfigDao::propose_update<STC, TransactionPublishOption::TransactionPublishOption>(&signer, new_config, 0);
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 87000000

//# run --signers bob
script {
    use StarcoinFramework::OnChainConfigDao;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Account;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Dao;
    use StarcoinFramework::TransactionPublishOption;
    fun vote(signer: signer) {
        let balance = Account::balance<STC>(Signer::address_of(&signer));
        let balance = Account::withdraw<STC>(&signer, balance/2);
        Dao::cast_vote<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(&signer, @alice, 0, balance, true);
    }
}


//# block --author 0x1 --timestamp 110000000

//# block --author 0x1 --timestamp 120000000

//# run --signers bob
script {
    use StarcoinFramework::OnChainConfigDao;
    use StarcoinFramework::TransactionPublishOption;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Account;
    use StarcoinFramework::Dao;
    fun queue_proposal(signer: signer) {
        let state = Dao::proposal_state<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(@alice, 0);
        assert!(state == 4, (state as u64));
        {
            let token = Dao::unstake_votes<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(&signer, @alice, 0);
            Account::deposit_to_self(&signer, token);
        };
        Dao::queue_proposal_action<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(@alice, 0);
        // ModifyDaoConfigProposal::execute<STC>(@alice, 0);
        let state = Dao::proposal_state<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(@alice, 0);
        assert!(state == 5, (state as u64));
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 130000000

//# run --signers bob
script {
    use StarcoinFramework::OnChainConfigDao;
    use StarcoinFramework::TransactionPublishOption;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Dao;
    fun execute_proposal_action(_signer: signer) {
        let state = Dao::proposal_state<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(@alice, 0);
        assert!(state == 6, (state as u64));
        OnChainConfigDao::execute<STC, TransactionPublishOption::TransactionPublishOption>(@alice, 0);
        assert!(!TransactionPublishOption::is_module_allowed(@StarcoinFramework), 401);
        assert!(TransactionPublishOption::is_script_allowed(@StarcoinFramework), 402);
    }
}
// check: EXECUTED

