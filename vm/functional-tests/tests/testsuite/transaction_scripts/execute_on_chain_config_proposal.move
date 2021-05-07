//! account: alice, 15925680000000000 0x1::STC::STC
//! account: bob, 15925680000000000 0x1::STC::STC

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: alice
//! args: false, false, 0
script {
    use 0x1::OnChainConfigScripts;

    fun main(account: signer,
             script_allowed: bool,
             module_publishing_allowed: bool,
             exec_delay: u64) {
        OnChainConfigScripts::propose_update_txn_publish_option(account, script_allowed, module_publishing_allowed, exec_delay);
    }
}
// check: gas_used
// check: 197129
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 87000000

//! new-transaction
//! sender: bob
//! args: {{alice}}, 0, true, 3981420001000000u128
script {
    use 0x1::DaoVoteScripts;
    use 0x1::STC::STC;
    use 0x1::OnChainConfigDao::OnChainConfigUpdate;
    use 0x1::TransactionPublishOption::TransactionPublishOption;

    fun main(account: signer,
            proposer_address: address,
            proposal_id: u64,
            agree: bool,
            votes: u128
        ) {
        DaoVoteScripts::cast_vote<STC, OnChainConfigUpdate<TransactionPublishOption>>(
            account,
            proposer_address,
            proposal_id,
            agree,
            votes);
    }
}
// check: gas_used
// check: 176139
// check: "Keep(EXECUTED)"


//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 110000000

//! new-transaction
//! sender: bob
//! args: {{alice}}, 0
script {
    use 0x1::Dao;
    use 0x1::STC::STC;
    use 0x1::OnChainConfigDao::OnChainConfigUpdate;
    use 0x1::TransactionPublishOption::TransactionPublishOption;

    fun main(_account: signer,
            proposer_address: address,
            proposal_id: u64,
        ) {
        Dao::queue_proposal_action<STC, OnChainConfigUpdate<TransactionPublishOption>>(
            proposer_address,
            proposal_id
        );
    }
}
// check: gas_used
// check: 54457
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! args: {{alice}}, 0
script {
    use 0x1::DaoVoteScripts;
    use 0x1::STC::STC;
    use 0x1::OnChainConfigDao::OnChainConfigUpdate;
    use 0x1::TransactionPublishOption::TransactionPublishOption;

    fun main(account: signer,
            proposer_address: address,
            proposal_id: u64,
        ) {
        DaoVoteScripts::unstake_vote<STC, OnChainConfigUpdate<TransactionPublishOption>>(
            account,
            proposer_address,
            proposal_id
        );
    }
}
// check: gas_used
// check: 118831
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 130000000

//! new-transaction
//! sender: alice
//! args: 0
script {
    use 0x1::OnChainConfigScripts;
    use 0x1::TransactionPublishOption::TransactionPublishOption;

    fun main(account: signer, proposal_id: u64) {
        OnChainConfigScripts::execute_on_chain_config_proposal<TransactionPublishOption>(
            account,
            proposal_id
        );
    }
}
// check: gas_used
// check: 115781
// check: "Keep(EXECUTED)"