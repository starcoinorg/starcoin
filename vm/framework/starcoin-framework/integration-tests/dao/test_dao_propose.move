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
// check: EXECUTED


//# run --signers alice


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(signer: signer) {
        dao_modify_config_proposal::propose<STC>(signer, 60 * 60 * 24 * 1000, 0, 50, 0, 0);
    }
}
// check: EXECUTED


//# run --signers alice


script {
    use starcoin_framework::dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao_modify_config_proposal;

    fun proposal_info(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 1, (state as u64));

        let (id, start_time, end_time, for_votes, against_votes)
            = dao::proposal_info<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice);

        assert!(id == 0, 101);
        assert!(start_time == 86460000, 102); // be consistent with genesis config
        assert!(end_time == 90060000, 103); // be consistent with genesis config
        assert!(for_votes == 0, 104);
        assert!(against_votes == 0, 104);
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 87000000


//# run --signers bob
// call cast_vote to stake some token


script {
    use starcoin_framework::dao;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun vote(signer: signer) {
        let balance = coin::balance<STC>(signer::address_of(&signer));
        let balance = coin::withdraw<STC>(&signer, balance / 2);
        dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, balance, true);
    }
}
// check: EXECUTED


//# run --signers bob
// call cast_vote again to stake more token


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;
    use starcoin_framework::dao;

    fun vote(signer: signer) {
        let balance = coin::balance<STC>(signer::address_of(&signer));
        let balance = coin::withdraw<STC>(&signer, balance / 2);
        dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, balance, true);
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 88000000


//# run --signers bob
// test revoke_vote


script {
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun check_state_and_revoke(account: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 2, (state as u64));
        let (_, pow) = dao::vote_of<STC>(signer::address_of(&account), @alice, 0);
        let token = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&account, @alice, 0, pow / 2);
        coin::deposit(signer::address_of(&account), token);
    }
}
// check: EXECUTED


//# run --signers bob


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun recast_vote(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 2, (state as u64));
        {
            let balance = coin::balance<STC>(signer::address_of(&signer));
            let balance = coin::withdraw<STC>(&signer, balance / 2);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0, balance, true);
        }
    }
}
// check: EXECUTED


//# run --signers bob
// test flip_vote


script {
    use starcoin_framework::dao;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun flip_vote(account: signer) {
        // flip
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&account, @alice, 0, false);
        {
            let balance = coin::balance<STC>(signer::address_of(&account));
            let balance = coin::withdraw<STC>(&account, balance / 2);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&account, @alice, 0, balance, false);
        };
        // revoke while 'against'
        {
            let (_, pow) = dao::vote_of<STC>(signer::address_of(&account), @alice, 0);
            let token = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                &account,
                @alice,
                0,
                pow / 10
            );
            coin::deposit(&account, token);
        };
        // flip back
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&account, @alice, 0, true);
        {
            let balance = coin::balance<STC>(signer::address_of(&account));
            let balance = coin::withdraw<STC>(&account, balance / 2);
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(&account, @alice, 0, balance, true);
        };
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 180000000


//# run --signers bob


script {
    use starcoin_framework::dao;
    use starcoin_framework::coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun queue_proposal(signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 4, (state as u64));
        {
            let token = dao::unstake_votes<STC, dao_modify_config_proposal::DaoConfigUpdate>(&signer, @alice, 0);
            coin::deposit(&signer, token);
        };
        dao::queue_proposal_action<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        // ModifyDaoConfigProposal::execute<STC>(@alice, 0);
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 5, (state as u64));
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 250000000


//# run --signers bob


script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun cleanup_proposal_should_fail(_signer: signer) {
        dao::destroy_terminated_proposal<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
    }
}
// check: "Keep(ABORTED { code: 359169"


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


//# block --author 0x1 --timestamp  310000000


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

