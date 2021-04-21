//! account: alice, 15925680000000000 0x1::STC::STC
//! account: bob, 15925680000000000 0x1::STC::STC

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: alice
//! args: 86400000, 0, 50u8, 0, 0
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;

    fun main(signer: signer,
            voting_delay: u64,
            voting_period: u64,
            voting_quorum_rate: u8,
            min_action_delay: u64,
            exec_delay: u64,) {
        ModifyDaoConfigProposal::propose<STC>(signer, voting_delay, voting_period, voting_quorum_rate, min_action_delay, exec_delay);
    }
}
// check: gas_used
// check: 198227
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
    use 0x1::ModifyDaoConfigProposal::DaoConfigUpdate;

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
    use 0x1::ModifyDaoConfigProposal::DaoConfigUpdate;

    fun main(_account: signer,
            proposer_address: address,
            proposal_id: u64,
        ) {
        Dao::queue_proposal_action<STC, DaoConfigUpdate>(
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
    use 0x1::ModifyDaoConfigProposal::DaoConfigUpdate;

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
// check: gas_used
// check: 118831
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 250000000

//! new-transaction
//! sender: bob
//! args: {{alice}}, 0
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;

    fun main(proposer_address: address, proposal_id: u64) {
        ModifyDaoConfigProposal::execute<STC>(
            proposer_address,
            proposal_id
        );
    }
}
// check: gas_used
// check: 163472
// check: "Keep(EXECUTED)"