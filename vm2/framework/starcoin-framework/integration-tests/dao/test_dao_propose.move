//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# block --author 0x1 --timestamp 86400000

//# run --signers StarcoinAssociation
script {
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::signer;

    fun transfer_some_token_to_alice_and_bob(association: &signer) {
        let stc_metadata = starcoin_coin::get_stc_fa_metadata();
        let balance = primary_fungible_store::balance(
            signer::address_of(association),
            starcoin_coin::get_stc_fa_metadata(),
        );
        primary_fungible_store::transfer(association, stc_metadata, @alice, balance / 4);
        primary_fungible_store::transfer(association, stc_metadata, @bob, balance / 4);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun propose(account: signer) {
        dao_modify_config_proposal::propose<STC>(
            account,
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
    use starcoin_std::debug;
    use starcoin_framework::dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao_modify_config_proposal;

    fun proposal_info(_signer: signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 1, (state as u64));

        let (id, start_time, end_time, for_votes, against_votes)
            = dao::proposal_info<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice);

        assert!(id == 0, 101);
        debug::print(&std::string::utf8(b"test_dao_proposal::proposal_info | Start time:"));
        debug::print(&start_time);
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
    use std::signer;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::dao;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun vote(bob: &signer) {
        // let balance = coin::balance<STC>(signer::address_of(&signer));
        // let balance = coin::withdraw<STC>(&signer, balance / 2);

        let stc_metadata = starcoin_coin::get_stc_fa_metadata();
        let balance = primary_fungible_store::balance(
            signer::address_of(bob),
            starcoin_coin::get_stc_fa_metadata(),
        );
        let fa = primary_fungible_store::withdraw(bob, stc_metadata, balance / 2);

        dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            bob,
            @alice,
            0,
            fa,
            true
        );
    }
}
// check: EXECUTED


//# run --signers bob
// call cast_vote again to stake more token


script {
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;
    use starcoin_framework::dao;

    fun vote(bob: &signer) {
        let stc_metadata = starcoin_coin::get_stc_fa_metadata();
        let balance = primary_fungible_store::balance(
            signer::address_of(bob),
            starcoin_coin::get_stc_fa_metadata(),
        );
        let fa = primary_fungible_store::withdraw(bob, stc_metadata, balance / 2);
        dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            bob,
            @alice,
            0,
            fa,
            true
        );
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 88000000

//# run --signers bob
// test revoke_vote
script {
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun check_state_and_revoke(bob: &signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 2, (state as u64));
        let (_, pow) = dao::vote_of<STC>(signer::address_of(bob), @alice, 0);

        let fa = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
            bob,
            @alice,
            0,
            pow
        );
        primary_fungible_store::deposit(signer::address_of(bob), fa)
    }
}
// check: EXECUTED


//# run --signers bob
script {
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_framework::signer;

    fun recast_vote(bob: &signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 2, (state as u64));
        {
            // let balance = coin::balance<STC>(signer::address_of(bob));
            // let balance = coin::withdraw<STC>(bob, balance / 2);

            let stc_metadata = starcoin_coin::get_stc_fa_metadata();
            let balance = primary_fungible_store::balance(
                signer::address_of(bob),
                starcoin_coin::get_stc_fa_metadata(),
            );
            let fa = primary_fungible_store::withdraw(bob, stc_metadata, balance / 2);

            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(bob, @alice, 0, fa, true);
        }
    }
}
// check: EXECUTED


//# run --signers bob
// test flip_vote
script {
    use std::signer;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::dao;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun flip_vote(bob: &signer) {
        let stc_metadata = starcoin_coin::get_stc_fa_metadata();

        // flip
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(bob, @alice, 0, false);
        {
            // let balance = coin::balance<STC>(signer::address_of(&account));
            // let balance = coin::withdraw<STC>(&account, balance / 2);

            let fa = primary_fungible_store::withdraw(
                bob,
                stc_metadata,
                primary_fungible_store::balance(
                    signer::address_of(bob),
                    starcoin_coin::get_stc_fa_metadata()
                ) / 2
            );

            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(bob, @alice, 0, fa, false);
        };
        // revoke while 'against'
        {
            let (_, pow) = dao::vote_of<STC>(signer::address_of(bob), @alice, 0);
            let fa = dao::revoke_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                bob,
                @alice,
                0,
                pow / 10
            );
            primary_fungible_store::deposit(signer::address_of(bob), fa);
        };

        // flip back
        dao::change_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(bob, @alice, 0, true);
        {
            // let balance = coin::balance<STC>(signer::address_of(bob));
            // let balance = coin::withdraw<STC>(bob, balance / 2);
            let fa = primary_fungible_store::withdraw(
                bob,
                stc_metadata,
                primary_fungible_store::balance(
                    signer::address_of(bob),
                    starcoin_coin::get_stc_fa_metadata()
                ) / 2
            );
            dao::cast_vote<STC, dao_modify_config_proposal::DaoConfigUpdate>(
                bob,
                @alice,
                0,
                fa,
                true
            );
        };
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 180000000


//# run --signers bob

script {
    use std::signer;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::dao;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun queue_proposal(bob: &signer) {
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);
        assert!(state == 4, (state as u64));
        {
            let fa = dao::unstake_votes<STC, dao_modify_config_proposal::DaoConfigUpdate>(bob, @alice, 0);
            primary_fungible_store::deposit(signer::address_of(bob), fa)
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
    use starcoin_std::debug;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun cleanup_proposal_should_fail(_signer: signer) {
        debug::print(&std::string::utf8(b"cleanup_proposal_should_fail | entered"));

        dao::destroy_terminated_proposal<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);

        debug::print(&std::string::utf8(b"cleanup_proposal_should_fail | exited"));
    }
}
// check: "Keep(ABORTED { code: 359169"


//# run --signers bob
script {
    use starcoin_std::debug;
    use starcoin_framework::dao_modify_config_proposal;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;

    fun execute_proposal_action(_signer: signer) {
        debug::print(&std::string::utf8(b"execute_proposal_action | entered"));
        let state = dao::proposal_state<STC, dao_modify_config_proposal::DaoConfigUpdate>(@alice, 0);

        debug::print(&std::string::utf8(b"execute_proposal_action | state = "));
        debug::print(&state);
        assert!(state == 6, (state as u64));
        dao_modify_config_proposal::execute<STC>(@alice, 0);

        debug::print(&std::string::utf8(b"execute_proposal_action | after dao_modify_config_proposal::execute "));

        assert!(dao::voting_delay<STC>() == 3600 * 24 * 1000, dao::voting_delay<STC>());
        assert!(dao::voting_quorum_rate<STC>() == 50, 1000);

        debug::print(&std::string::utf8(b"execute_proposal_action | exited "));
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

