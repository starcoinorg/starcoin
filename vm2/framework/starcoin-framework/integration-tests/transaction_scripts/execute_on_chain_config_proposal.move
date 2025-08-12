//# init -n dev

//# faucet --addr alice --amount 159256800000

//# faucet --addr bob --amount 49814200010000000

//# block --author 0x1 --timestamp 86400000


//# run --signers alice --args false --args false --args 0
script {
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::transaction_publish_option;

    fun main(account: signer,
             script_allowed: bool,
             module_publishing_allowed: bool,
             exec_delay: u64) {
        let txn_publish_option = transaction_publish_option::new_transaction_publish_option(
            script_allowed,
            module_publishing_allowed
        );
        on_chain_config_dao::propose_update<STC, transaction_publish_option::TransactionPublishOption>(
            &account,
            txn_publish_option,
            exec_delay
        );
    }
}
//# block --author 0x1 --timestamp 87000000

//# run --signers bob --args @alice --args 0 --args true --args 39814200010000000u128
script {
    use starcoin_framework::dao_vote_scripts;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::on_chain_config_dao::OnChainConfigUpdate;
    use starcoin_framework::transaction_publish_option::TransactionPublishOption;

    fun main(account: signer,
             proposer_address: address,
             proposal_id: u64,
             agree: bool,
             votes: u128
    ) {
        dao_vote_scripts::cast_vote<STC, OnChainConfigUpdate<TransactionPublishOption>>(
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
    use starcoin_framework::dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::on_chain_config_dao::OnChainConfigUpdate;
    use starcoin_framework::transaction_publish_option::TransactionPublishOption;

    fun main(
        _account: signer,
        proposer_address: address,
        proposal_id: u64,
    ) {
        dao::queue_proposal_action<STC, OnChainConfigUpdate<TransactionPublishOption>>(
            proposer_address,
            proposal_id
        );
    }
}

//# run --signers bob --args @alice --args 0
script {
    use starcoin_framework::transaction_publish_option::TransactionPublishOption;
    use starcoin_framework::on_chain_config_dao;
    use starcoin_framework::dao_vote_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer,
             proposer_address: address,
             proposal_id: u64,
    ) {
        dao_vote_scripts::unstake_vote<STC, on_chain_config_dao::OnChainConfigUpdate<TransactionPublishOption>>(
            account,
            proposer_address,
            proposal_id
        );
    }
}

//# block --author 0x1 --timestamp 250000000

//# run --signers alice  --args 0
script {
    use std::signer;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::transaction_publish_option::TransactionPublishOption;
    use starcoin_framework::on_chain_config_dao;

    fun main(account: signer, proposal_id: u64) {
        on_chain_config_dao::execute<STC, TransactionPublishOption>(
            signer::address_of(&account),
            proposal_id
        );
    }
}
