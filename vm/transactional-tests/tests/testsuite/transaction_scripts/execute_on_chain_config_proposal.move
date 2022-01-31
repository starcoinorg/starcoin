//# init -n dev

//# faucet --addr alice --amount 159256800000

//# faucet --addr bob --amount 49814200010000000

//# block --author 0x1 --timestamp 86400000


//# run --signers alice --args false --args false --args 0
script {
    use StarcoinFramework::OnChainConfigScripts;

    fun main(account: signer,
             script_allowed: bool,
             module_publishing_allowed: bool,
             exec_delay: u64) {
        OnChainConfigScripts::propose_update_txn_publish_option(account, script_allowed, module_publishing_allowed, exec_delay);
    }
}
//# block --author 0x1 --timestamp 87000000

//# run --signers bob --args @alice --args 0 --args true --args 39814200010000000u128
script {
    use StarcoinFramework::DaoVoteScripts;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::OnChainConfigDao::OnChainConfigUpdate;
    use StarcoinFramework::TransactionPublishOption::TransactionPublishOption;

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


//# block --author 0x1 --timestamp 110000000

//# run --signers bob --args @alice --args 0
script {
    use StarcoinFramework::Dao;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::OnChainConfigDao::OnChainConfigUpdate;
    use StarcoinFramework::TransactionPublishOption::TransactionPublishOption;

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

//# run --signers bob --args @alice --args 0
script {
    use StarcoinFramework::DaoVoteScripts;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::OnChainConfigDao::OnChainConfigUpdate;
    use StarcoinFramework::TransactionPublishOption::TransactionPublishOption;

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

//# block --author 0x1 --timestamp 250000000

//# run --signers alice  --args 0

script {
    use StarcoinFramework::OnChainConfigScripts;
    use StarcoinFramework::TransactionPublishOption::TransactionPublishOption;

    fun main(account: signer, proposal_id: u64) {
        OnChainConfigScripts::execute_on_chain_config_proposal<TransactionPublishOption>(
            account,
            proposal_id
        );
    }
}
