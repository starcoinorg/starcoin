//! account: alice
//! account: bob
//! account: carol

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

    fun transfer_some_token_to_alice_and_bob(signer: signer) {
        let balance = Account::balance<STC>(Signer::address_of(&signer));
        Account::pay_from<STC>(&signer, {{alice}}, balance / 4);
        Account::pay_from<STC>(&signer, {{bob}}, balance / 4);
        Account::pay_from<STC>(&signer, {{carol}}, balance / 4);
    }
}
// check: EXECUTED
// voting_quorum_rate should less or equal than 100
//! new-transaction
//! sender: alice
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    fun propose(signer: signer) {
        ModifyDaoConfigProposal::propose<STC>(signer, 60 * 60 * 24 * 1000, 0, 101, 0, 0);
    }
}
// check: "Keep(ABORTED { code: 102919"

//! new-transaction
//! sender: alice
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    fun propose(signer: signer) {
        ModifyDaoConfigProposal::propose<STC>(signer, 60 * 60 * 24 * 1000, 0, 50, 0, 0);
    }
}
// check: EXECUTED


//! new-transaction
//! sender: bob
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    fun propose(signer: signer) {
        ModifyDaoConfigProposal::propose<STC>(signer, 60 * 60 * 24 * 1000, 0, 50, 0, 0);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    fun propose(signer: signer) {
        ModifyDaoConfigProposal::propose<STC>(signer, 60 * 60 * 24 * 1000, 0, 50, 0, 0);
    }
}
// check: RESOURCE_ALREADY_EXISTS

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 87000000

//! new-transaction
//! sender: alice
// call cast_vote to stake some token
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Account;
    fun vote(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{bob}}, 1);
        assert(state == 2, (state as u64));
        {
            let balance = Account::withdraw<STC>(&signer, 10); // less than quorum_votes
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{bob}}, 1, balance, true);
        }
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
// call cast_vote to stake some token
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::Dao;
    fun vote(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 2, (state as u64));
        {
            let balance = Account::balance<STC>(Signer::address_of(&signer));
            let balance = Account::withdraw<STC>(&signer, balance / 2);
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, balance, true);
        }
    }
}
// check: EXECUTED


//! new-transaction
//! sender: bob
// vote 'agree' votes on 'against' voting
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::Dao;
    fun vote(signer: signer) {
        // flip
        Dao::change_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, false);

        {
            let balance = Account::balance<STC>(Signer::address_of(&signer));
            let balance = Account::withdraw<STC>(&signer, balance / 2);
            // ERR_VOTE_STATE_MISMATCH
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, balance, true);
        };
        // flip back
        Dao::change_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, true);
    }
}
// check: "Keep(ABORTED { code: 360449"

//! new-transaction
//! sender: bob
// cast a vote with wrong proposer, already vote others
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun cast(signer: signer) {
        {
            let balance = Account::balance<STC>(Signer::address_of(&signer));
            let balance = Account::withdraw<STC>(&signer, balance / 2);
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{bob}}, 1, balance, true);
        }
    }
}
// check: "Keep(ABORTED { code: 360967"

//! new-transaction
//! sender: bob
// cast a vote with wrong proposal id
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun cast(signer: signer) {
        {
            let balance = Account::balance<STC>(Signer::address_of(&signer));
            let balance = Account::withdraw<STC>(&signer, balance / 2);
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 1, balance, true);
        }
    }
}
// check: "Keep(ABORTED { code: 359431"

//! new-transaction
//! sender: bob
// revoke a vote with wrong proposer
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun check_state_and_revoke(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 2, (state as u64));
        let (_, pow) = Dao::vote_of<STC>(Signer::address_of(&signer), {{alice}}, 0);
        let token = Dao::revoke_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{bob}}, 1, pow / 2); // proposer should be alice
        Account::deposit_to_self(&signer, token);
    }
}
// check: "Keep(ABORTED { code: 359687"

//! new-transaction
//! sender: bob
// revoke a vote with wrong proposal id
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun check_state_and_revoke(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 2, (state as u64));
        let (_, pow) = Dao::vote_of<STC>(Signer::address_of(&signer), {{alice}}, 0);
        let token = Dao::revoke_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 1, pow / 2); // proposal id should be 0
        Account::deposit_to_self(&signer, token);
    }
}
// check: "Keep(ABORTED { code: 359431"

//! new-transaction
//! sender: bob
// flip_vote failed, wrong proposer
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun flip_vote(signer: signer) {
        // flip
        Dao::change_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{bob}}, 1, false);
    }
}
// check: "Keep(ABORTED { code: 359687"

//! new-transaction
//! sender: bob
// flip_vote, flip 'agree' vote with 'agree', do nothing
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun flip_vote(signer: signer) {
        // flip
        Dao::change_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, true);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
// flip_vote failed, wrong id
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun flip_vote(signer: signer) {
        // flip
        Dao::change_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 1, false);
    }
}
// check: "Keep(ABORTED { code: 359431"

//! new-transaction
//! sender: bob
// unstake_votes failed, wrong state, proposal is still active
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Account;
    fun unstake_votes(signer: signer) {
        let coin = Dao::unstake_votes<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{bob}}, 1);
        Account::deposit_to_self(&signer, coin);
    }
}
// check: "Keep(ABORTED { code: 359169"

//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 250000000

//! new-transaction
//! sender: bob
// check state
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun check_state_and_revoke(_signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 4, (state as u64));
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{bob}}, 1);
        assert(state == 3, (state as u64));
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
// unstake_votes failed, wrong proposer
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Account;
    fun unstake_votes(signer: signer) {
        let coin = Dao::unstake_votes<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{bob}}, 1);
        Account::deposit_to_self(&signer, coin);
    }
}
// check: "Keep(ABORTED { code: 359682"

//! new-transaction
//! sender: bob
// can't cast vote in the state other than ACTIVE
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun check_state_and_revoke(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 4, (state as u64));
        {
            let balance = Account::balance<STC>(Signer::address_of(&signer));
            let balance = Account::withdraw<STC>(&signer, balance);
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, balance, true);
        }
    }
}
// check: "Keep(ABORTED { code: 359169"

//! new-transaction
//! sender: bob
// can't change vote in the state other than ACTIVE
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun check_state_and_revoke(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 4, (state as u64));
        Dao::change_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, false);
    }
}
// check: "Keep(ABORTED { code: 359169"

//! new-transaction
//! sender: bob

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun check_state_and_revoke(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 4, (state as u64));
        let (_, pow) = Dao::vote_of<STC>(Signer::address_of(&signer), {{alice}}, 0);
        let token = Dao::revoke_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 0, pow / 2);
        Account::deposit_to_self(&signer, token);
    }
}
// check: "Keep(ABORTED { code: 359169"


//! new-transaction
//! sender: bob

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun queue_proposal(_signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 4, (state as u64));

        Dao::queue_proposal_action<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        // ModifyDaoConfigProposal::execute<STC>({{alice}}, 0);
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 5, (state as u64));
    }
}
// check: EXECUTED


//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 260000000

//! new-transaction
//! sender: bob

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun execute_proposal_action(_signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 6, (state as u64));
        ModifyDaoConfigProposal::execute<STC>({{alice}}, 0);
        assert(Dao::voting_delay<STC>()==3600 * 24 * 1000, Dao::voting_delay<STC>());
        assert(Dao::voting_quorum_rate<STC>() == 50, 1000);
    }
}
// check: EXECUTED


//! block-prologue
//! author: genesis
//! block-number: 5
//! block-time: 310000000


//! new-transaction
//! sender: bob

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun cleanup_proposal(_signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
        assert(state == 7, (state as u64));
        Dao::destroy_terminated_proposal<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 0);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun cleanup_proposal(_signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{bob}}, 1);
        assert(state == 3, (state as u64));
        //ERR_PROPOSAL_STATE_INVALID
        Dao::extract_proposal_action<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{bob}}, 1);
    }
}
// check: "Keep(ABORTED { code: 359169"

//! new-transaction
//! sender: alice

script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun cleanup_proposal(_signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{bob}}, 1);
        assert(state == 3, (state as u64));
        Dao::destroy_terminated_proposal<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{bob}}, 1);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
// alice proposes a new proposal, the proposal_id is 2.
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    fun propose(signer: signer) {
        ModifyDaoConfigProposal::propose<STC>(signer, 60 * 60 * 24 * 1000, 0, 50, 0, 0);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 6
//! block-time: 400000000

//! new-transaction
//! sender: bob
// cast_vote will be failed, already vote others
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::Dao;
    fun vote(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 2);
        assert(state == 2, (state as u64));
        {
            let balance = Account::balance<STC>(Signer::address_of(&signer));
            let balance = Account::withdraw<STC>(&signer, balance / 2);
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 2, balance, true);
        }
    }
}
// check: "Keep(ABORTED { code: 360967"

//! new-transaction
//! sender: bob
// revoke vote failed, alice has already proposed new proposal with id(2)
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Signer;
    use 0x1::Account;
    fun check_state_and_revoke(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 2);
        assert(state == 2, (state as u64));
        let (_, pow) = Dao::vote_of<STC>(Signer::address_of(&signer), {{alice}}, 0);
        let token = Dao::revoke_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 2, pow / 2);
        Account::deposit_to_self(&signer, token);
    }
}
// check: "Keep(ABORTED { code: 360967"


//! new-transaction
//! sender: bob
// flip_vote failed, alice has already proposed new proposal with id(2)
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    fun flip_vote(signer: signer) {
        // flip
        Dao::change_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 2, false);
    }
}
// check: "Keep(ABORTED { code: 360967"

//! new-transaction
//! sender: carol
// call cast_vote to stake some token
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::Dao;
    fun vote(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 2);
        assert(state == 2, (state as u64));
        {
            let balance = Account::balance<STC>(Signer::address_of(&signer));
            let balance = Account::withdraw<STC>(&signer, balance / 2);
            Dao::cast_vote<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 2, balance, false);
        }
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 7
//! block-time: 600000000


//! new-transaction
//! sender: bob
// unstake_votes failed, wrong proposal id
script {
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::STC::STC;
    use 0x1::Dao;
    use 0x1::Account;
    fun unstake_votes(signer: signer) {
        let state = Dao::proposal_state<STC, ModifyDaoConfigProposal::DaoConfigUpdate>({{alice}}, 2);
        assert(state == 3, (state as u64));
        // bob should unstake proposal [{{alice}}, 0]
        let coin = Dao::unstake_votes<STC, ModifyDaoConfigProposal::DaoConfigUpdate>(&signer, {{alice}}, 2);
        Account::deposit_to_self(&signer, coin);
    }
}

// check: "Keep(ABORTED { code: 360967"




