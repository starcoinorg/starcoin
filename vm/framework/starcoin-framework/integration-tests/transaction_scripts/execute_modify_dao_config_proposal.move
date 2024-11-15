//# init -n dev


//# faucet --addr alice --amount 159256800000

//# faucet --addr bob --amount 49814200010000000

//# block --author 0x1 --timestamp 86400000


//# run --signers alice --args 86400000 --args 0 --args 50u8 --args 0 --args 0
script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun main(signer: signer,
            voting_delay: u64,
            voting_period: u64,
            voting_quorum_rate: u8,
            min_action_delay: u64,
            exec_delay: u64,) {
        dao_modify_config_proposal::propose<STC>(signer, voting_delay, voting_period, voting_quorum_rate, min_action_delay, exec_delay);
    }
}

//# block --author 0x1 --timestamp 87000000

//# run --signers bob --args @alice --args 0 --args true --args 39814200010000000u128
script {
    use starcoin_framework::DaoVoteScripts;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao_modify_config_proposal::DaoConfigUpdate;

    fun main(account: signer,
            proposer_address: address,
            proposal_id: u64,
            agree: bool,
            votes: u128
        ) {
        DaoVoteScripts::cast_vote<STC, DaoConfigUpdate>(
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
    use starcoin_framework::dao_modify_config_proposal::DaoConfigUpdate;

    fun main(_account: signer,
            proposer_address: address,
            proposal_id: u64,
        ) {
        dao::queue_proposal_action<STC, DaoConfigUpdate>(
            proposer_address,
            proposal_id
        );
    }
}


//# run --signers bob --args @alice --args 0
script {
    use starcoin_framework::DaoVoteScripts;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao_modify_config_proposal::DaoConfigUpdate;

    fun main(account: signer,
             proposer_address: address,
             proposal_id: u64,
    ) {
        DaoVoteScripts::unstake_vote<STC, DaoConfigUpdate>(
            account,
            proposer_address,
            proposal_id
        );
    }
}
//# block --author 0x1 --timestamp 250000000

//# run --signers bob --args @alice --args 0

script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun main(proposer_address: address, proposal_id: u64) {
        dao_modify_config_proposal::execute<STC>(
            proposer_address,
            proposal_id
        );
    }
}
