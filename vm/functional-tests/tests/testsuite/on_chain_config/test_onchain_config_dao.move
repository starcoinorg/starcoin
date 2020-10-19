//! account: alice
//! account: bob

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: association
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Signer;

    fun transfer_some_token_to_alice_and_bob(signer: &signer) {
        let balance = Account::balance<STC>(Signer::address_of(signer));
        Account::pay_from<STC>(signer, {{alice}}, balance / 4);
        Account::pay_from<STC>(signer, {{bob}}, balance / 4);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::OnChainConfigDao;
    use 0x1::TransactionPublishOption;
    use 0x1::STC::STC;
    use 0x1::Vector;
    fun propose(signer: &signer) {
        let new_config = TransactionPublishOption::new_transaction_publish_option(Vector::empty(), false);
        OnChainConfigDao::propose_update<STC, TransactionPublishOption::TransactionPublishOption>(signer, new_config, 0);
    }
}
// check: EXECUTED


//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 87000000


//! new-transaction
//! sender: bob

script {
    use 0x1::OnChainConfigDao;
    use 0x1::STC::STC;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::Dao;
    use 0x1::TransactionPublishOption;
    fun vote(signer: &signer) {
        let balance = Account::balance<STC>(Signer::address_of(signer));
        let balance = Account::withdraw<STC>(signer, balance);
        Dao::cast_vote<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(signer, {{alice}}, 0, balance, true);
    }
}
// check: EXECUTED


//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 110000000



//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 120000000

//! new-transaction
//! sender: bob

script {
    use 0x1::OnChainConfigDao;
    use 0x1::TransactionPublishOption;
    use 0x1::STC::STC;
    use 0x1::Account;
    use 0x1::Dao;
    fun queue_proposal(signer: &signer) {
        let state = Dao::proposal_state<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>({{alice}}, 0);
        assert(state == 4, (state as u64));
        {
            let token = Dao::unstake_votes<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>(signer, {{alice}}, 0);
            Account::deposit_to_self(signer, token);
        };
        Dao::queue_proposal_action<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>({{alice}}, 0);
        // ModifyDaoConfigProposal::execute<STC>({{alice}}, 0);
        let state = Dao::proposal_state<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>({{alice}}, 0);
        assert(state == 5, (state as u64));
    }
}
// check: EXECUTED


//! block-prologue
//! author: genesis
//! block-number: 5
//! block-time: 130000000



//! new-transaction
//! sender: bob

script {
    use 0x1::OnChainConfigDao;
    use 0x1::TransactionPublishOption;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun execute_proposal_action(_signer: &signer) {
        let state = Dao::proposal_state<STC, OnChainConfigDao::OnChainConfigUpdate<TransactionPublishOption::TransactionPublishOption>>({{alice}}, 0);
        assert(state == 6, (state as u64));
        OnChainConfigDao::execute<STC, TransactionPublishOption::TransactionPublishOption>({{alice}}, 0);
        assert(!TransactionPublishOption::is_module_allowed({{genesis}}), 401);
        assert(TransactionPublishOption::is_script_allowed({{genesis}},&x"010"), 402);
    }
}
// check: EXECUTED

