//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol

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
        coin::transfer<STC>(&signer, @carol, balance / 4);
    }
}
// check: EXECUTED
// voting_quorum_rate should less or equal than 100

//# run --signers alice
script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(signer: signer) {
        dao_modify_config_proposal::propose<STC>(
            signer,
            60 * 60 * 24 * 1000,
            0,
            101,
            0,
            0
        );
    }
}
// check: "Keep(ABORTED { code: 102919"


//# run --signers alice
script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(signer: signer) {
        dao_modify_config_proposal::propose<STC>(
            signer,
            60 * 60 * 24 * 1000,
            0,
            50,
            0,
            0
        );
    }
}
// check: EXECUTED


//# run --signers bob
script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(signer: signer) {
        dao_modify_config_proposal::propose<STC>(
            signer,
            60 * 60 * 24 * 1000,
            0,
            50,
            0,
            0
        );
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(signer: signer) {
        dao_modify_config_proposal::propose<STC>(signer,
            60 * 60 * 24 * 1000,
            0,
            50,
            0,
            0
        );
    }
}
// check: RESOURCE_ALREADY_EXISTS


//# block --author 0x1 --timestamp 87000000

//# run --signers alice
// call cast_vote to stake some token
script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun vote(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            @bob,
            1
        );
        assert!(state == 2, (state as u64));
        {
            let balance = coin::withdraw<STC>(&signer, 10); // less than quorum_votes
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                &signer,
                @bob,
                1,
                balance,
                true
            );
        }
    }
}
// check: EXECUTED


//# run --signers bob
// call cast_vote to stake some token


script {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun vote(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 2, (state as u64));
        {
            let balance = coin::balance<STC>(signer::address_of(&signer));
            let balance = coin::withdraw<STC>(&signer, balance / 2);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                &signer,
                @alice,
                0,
                balance,
                true
            );
        }
    }
}
// check: EXECUTED


//# run --signers bob
// vote 'agree' votes on 'against' voting


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;
    use starcoin_framework::dao;

    fun vote(signer: signer) {
        // flip
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, false);

        {
            let balance = coin::balance<STC>(signer::address_of(&signer));
            let balance = coin::withdraw<STC>(&signer, balance / 2);
            // ERR_VOTE_STATE_MISMATCH
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, balance, true);
        };
        // flip back
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, true);
    }
}
// check: "Keep(ABORTED { code: 360449"


//# run --signers bob
// cast a vote with wrong proposer, already vote others


script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun cast(signer: signer) {
        {
            let balance = coin::balance<STC>(signer::address_of(&signer));
            let balance = coin::withdraw<STC>(&signer, balance / 2);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @bob, 1, balance, true);
        }
    }
}
// check: "Keep(ABORTED { code: 360967"


//# run --signers bob
// cast a vote with wrong proposal id


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun cast(signer: signer) {
        let balance = coin::balance<STC>(signer::address_of(&signer));
        let balance = coin::withdraw<STC>(&signer, balance / 2);
        dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            &signer,
            @alice,
            1,
            balance,
            true
        );
    }
}
// check: "Keep(ABORTED { code: 359431"


//# run --signers bob
// revoke a vote with wrong proposer


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun check_state_and_revoke(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 2, (state as u64));
        let (_, pow) = dao::vote_of<STC>(
            signer::address_of(&signer),
            @alice,
            0
        );
        let token = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            &signer,
            @bob,
            1,
            pow / 2
        ); // proposer should be alice
        coin::deposit(signer::address_of(&signer), token);
    }
}
// check: "Keep(ABORTED { code: 359687"


//# run --signers bob
// revoke a vote with wrong proposal id


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun check_state_and_revoke(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 2, (state as u64));
        let (_, pow) = dao::vote_of<STC>(signer::address_of(&signer), @alice, 0);
        let token = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            &signer,
            @alice,
            1,
            pow / 2
        ); // proposal id should be 0
        coin::deposit(signer::address_of(&signer), token);
    }
}
// check: "Keep(ABORTED { code: 359431"


//# run --signers bob
// flip_vote failed, wrong proposer


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun flip_vote(signer: signer) {
        // flip
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @bob, 1, false);
    }
}
// check: "Keep(ABORTED { code: 359687"


//# run --signers bob
// flip_vote, flip 'agree' vote with 'agree', do nothing


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun flip_vote(signer: signer) {
        // flip
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, true);
    }
}
// check: EXECUTED


//# run --signers bob
// flip_vote failed, wrong id

script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun flip_vote(signer: signer) {
        // flip
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            &signer, @alice, 1, false);
    }
}
// check: "Keep(ABORTED { code: 359431"


//# run --signers bob
// unstake_votes failed, wrong state, proposal is still active


script {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun unstake_votes(signer: signer) {
        let coin = dao::unstake_votes<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @bob, 1);
        coin::deposit(signer::address_of(&signer), coin);
    }
}
// check: "Keep(ABORTED { code: 359169"


//# block --author 0x1 --timestamp 250000000

//# run --signers bob
// check state


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun check_state_and_revoke(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            @alice,
            0
        );
        assert!(state == 4, (state as u64));
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            @bob,
            1
        );
        assert!(state == 3, (state as u64));
    }
}
// check: EXECUTED


//# run --signers bob
// unstake_votes failed, wrong proposer


script {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun unstake_votes(signer: signer) {
        let coin = dao::unstake_votes<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @bob, 1);
        coin::deposit(signer::address_of(&signer), coin);
    }
}
// check: "Keep(ABORTED { code: 359682"


//# run --signers bob
// can't cast vote in the state other than ACTIVE


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun check_state_and_revoke(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 4, (state as u64));
        {
            let balance = coin::balance<STC>(signer::address_of(&signer));
            let balance = coin::withdraw<STC>(&signer, balance);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                &signer, @alice, 0, balance, true);
        }
    }
}
// check: "Keep(ABORTED { code: 359169"


//# run --signers bob
// can't change vote in the state other than ACTIVE


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun check_state_and_revoke(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 4, (state as u64));
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, false);
    }
}
// check: "Keep(ABORTED { code: 359169"


//# run --signers bob


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun check_state_and_revoke(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 4, (state as u64));
        let (_, pow) = dao::vote_of<STC>(signer::address_of(&signer), @alice, 0);
        let token = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, pow / 2);
        coin::deposit(signer::address_of(&signer), token);
    }
}
// check: "Keep(ABORTED { code: 359169"


//# run --signers bob


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun queue_proposal(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 4, (state as u64));

        dao::queue_proposal_action<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        // ModifyDaoConfigProposal::execute<STC>(@alice, 0);
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 5, (state as u64));
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 260000000

//# run --signers bob


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun execute_proposal_action(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 6, (state as u64));
        dao_modify_config_proposal::execute<STC>(@alice, 0);
        assert!(dao::voting_delay<STC>() == 3600 * 24 * 1000, dao::voting_delay<STC>());
        assert!(dao::voting_quorum_rate<STC>() == 50, 1000);
    }
}
// check: EXECUTED


//! block-prologue
//! author: genesis
//! block-number: 5
//! block-time: 310000000

//# block --author 0x1 --timestamp 310000000

//# run --signers bob


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun cleanup_proposal(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 7, (state as u64));
        dao::destroy_terminated_proposal<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
    }
}
// check: EXECUTED


//# run --signers alice


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun cleanup_proposal(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@bob, 1);
        assert!(state == 3, (state as u64));
        //ERR_PROPOSAL_STATE_INVALID
        dao::extract_proposal_action<STC, dao_modify_config_proposal::DaoConfigUpdate>(@bob, 1);
    }
}
// check: "Keep(ABORTED { code: 359169"


//# run --signers alice


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun cleanup_proposal(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@bob, 1);
        assert!(state == 3, (state as u64));
        dao::destroy_terminated_proposal<STC, dao_modify_config_proposal::DaoConfigUpdate>(@bob, 1);
    }
}
// check: EXECUTED


//# run --signers alice
// alice proposes a new proposal, the proposal_id is 2.


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(signer: signer) {
        dao_modify_config_proposal::propose<STC>(
            signer, 60 * 60 * 24 * 1000, 0, 50, 0, 0);
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 400000000


//# run --signers bob
// cast_vote will be failed, already vote others


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;
    use starcoin_framework::dao;

    fun vote(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 2);
        assert!(state == 2, (state as u64));
        {
            let balance = coin::balance<STC>(signer::address_of(&signer));
            let balance = coin::withdraw<STC>(&signer, balance / 2);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                &signer, @alice, 2, balance, true);
        }
    }
}
// check: "Keep(ABORTED { code: 360967"


//# run --signers bob
// revoke vote failed, alice has already proposed new proposal with id(2)


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun check_state_and_revoke(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 2);
        assert!(state == 2, (state as u64));
        let (_, pow) = dao::vote_of<STC>(signer::address_of(&signer), @alice, 0);
        let token = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 2, pow / 2);
        coin::deposit(signer::address_of(&signer), token);
    }
}
// check: "Keep(ABORTED { code: 360967"


//# run --signers bob
// flip_vote failed, alice has already proposed new proposal with id(2)


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun flip_vote(signer: signer) {
        // flip
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            &signer, @alice, 2, false);
    }
}
// check: "Keep(ABORTED { code: 360967"


//# run --signers carol
// call cast_vote to stake some token


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;
    use starcoin_framework::dao;

    fun vote(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 2);
        assert!(state == 2, (state as u64));
        {
            let balance = coin::balance<STC>(signer::address_of(&signer));
            let balance = coin::withdraw<STC>(&signer, balance / 2);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                &signer, @alice, 2, balance, false);
        }
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 600000000


//# run --signers bob
// unstake_votes failed, wrong proposal id


script {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun unstake_votes(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            @alice, 2
        );
        assert!(state == 3, (state as u64));
        // bob should unstake proposal [@alice, 0]
        let coin = dao::unstake_votes<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            &signer, @alice, 2);
        coin::deposit(signer::address_of(&signer), coin);
    }
}

// check: "Keep(ABORTED { code: 360967"




