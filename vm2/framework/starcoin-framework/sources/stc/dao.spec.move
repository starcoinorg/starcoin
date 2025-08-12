spec starcoin_framework::dao {

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    spec DaoConfig {
        invariant voting_quorum_rate > 0 && voting_quorum_rate <= 100;
        invariant voting_delay > 0;
        invariant voting_period > 0;
        invariant min_action_delay > 0;
    }

    spec plugin {
        use starcoin_framework::signer;

        let sender = signer::address_of(signer);
        aborts_if sender != @0x2;

        include NewDaoConfigParamSchema<TokenT>;

        include on_chain_config::PublishNewConfigAbortsIf<DaoConfig<TokenT>> { account: signer };

        aborts_if exists<DaoGlobalInfo<TokenT>>(sender);
    }

    spec schema RequirePluginDao<TokenT> {
        let token_addr = @0x2;
        aborts_if !exists<DaoGlobalInfo<TokenT>>(token_addr);
        aborts_if !exists<on_chain_config::Config<DaoConfig<TokenT>>>(token_addr);
    }

    spec schema AbortIfDaoInfoNotExist<TokenT> {
        let token_addr = @0x2;
        aborts_if !exists<DaoGlobalInfo<TokenT>>(token_addr);
    }

    spec schema AbortIfDaoConfigNotExist<TokenT> {
        let token_addr = @0x2;
        aborts_if !exists<on_chain_config::Config<DaoConfig<TokenT>>>(token_addr);
    }

    spec schema AbortIfTimestampNotExist {
        use starcoin_framework::system_addresses;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
    }

    spec module {
        apply
            AbortIfDaoInfoNotExist<TokenT>
            to
            generate_next_proposal_id<TokenT>;

        apply
            AbortIfDaoConfigNotExist<TokenT>
            to
            get_config<TokenT>,
            voting_delay<TokenT>,
            voting_period<TokenT>,
            voting_quorum_rate<TokenT>,
            min_action_delay<TokenT>,
            quorum_votes<TokenT>;
    }

    spec new_dao_config {
        include NewDaoConfigParamSchema<TokenT>;
    }

    spec schema NewDaoConfigParamSchema<TokenT> {
        voting_delay: u64;
        voting_period: u64;
        voting_quorum_rate: u8;
        min_action_delay: u64;

        aborts_if voting_delay == 0;
        aborts_if voting_period == 0;
        aborts_if voting_quorum_rate == 0 || voting_quorum_rate > 100;
        aborts_if min_action_delay == 0;
    }

    spec propose {
        use starcoin_framework::system_addresses;
        use starcoin_framework::signer;

        pragma verify = false;
        let proposer = signer::address_of(signer);

        include GenerateNextProposalIdSchema<TokenT>;

        pragma addition_overflow_unchecked = true; // start_time calculation

        include AbortIfDaoConfigNotExist<TokenT>;
        include AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());

        aborts_if action_delay > 0 && action_delay < spec_dao_config<TokenT>().min_action_delay;
        include CheckQuorumVotes<TokenT>;

        let sender = signer::address_of(signer);
        aborts_if exists<Proposal<TokenT, ActionT>>(sender);
        modifies global<DaoGlobalInfo<TokenT>>(@0x2);

        ensures exists<Proposal<TokenT, ActionT>>(sender);
    }

    spec schema CheckVoteOnCast<TokenT, ActionT> {
        proposal_id: u64;
        agree: bool;
        voter: address;
        stake_value: u128;
        let vote = global<Vote<TokenT>>(voter);
        aborts_if vote.id != proposal_id;
        aborts_if vote.agree != agree;
        aborts_if vote.stake.value + stake_value > MAX_U128;
    }

    spec cast_vote {
        pragma addition_overflow_unchecked = true;

        include AbortIfDaoInfoNotExist<TokenT>;

        let expected_states = vec(ACTIVE);
        include CheckProposalStates<TokenT, ActionT> { expected_states };
        let sender = signer::address_of(signer);
        let vote_exists = exists<Vote<TokenT>>(sender);
        include vote_exists ==> CheckVoteOnCast<TokenT, ActionT> {
            voter: sender,
            proposal_id: proposal_id,
            agree: agree,
            stake_value: stake.value,
        };

        modifies global<Proposal<TokenT, ActionT>>(proposer_address);
        ensures !vote_exists ==> global<Vote<TokenT>>(sender).stake.value == stake.value;
    }


    spec do_cast_vote {
        pragma addition_overflow_unchecked = true;
        aborts_if vote.stake.value + stake.value > MAX_U128;
        ensures vote.stake.value == old(vote).stake.value + stake.value;
        ensures vote.agree ==> old(proposal).for_votes + stake.value == proposal.for_votes;
        ensures vote.agree ==> old(proposal).against_votes == proposal.against_votes;
        ensures !vote.agree ==> old(proposal).against_votes + stake.value == proposal.against_votes;
        ensures !vote.agree ==> old(proposal).for_votes == proposal.for_votes;
    }

    spec schema CheckVoteOnProposal<TokenT> {
        vote: Vote<TokenT>;
        proposer_address: address;
        proposal_id: u64;

        aborts_if vote.id != proposal_id;
        aborts_if vote.proposer != proposer_address;
    }

    spec schema CheckChangeVote<TokenT, ActionT> {
        vote: Vote<TokenT>;
        proposer_address: address;
        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        include AbortIfDaoInfoNotExist<TokenT>;
        include CheckFlipVote<TokenT, ActionT> { my_vote: vote, proposal };
    }
    spec change_vote {
        pragma verify = false;
        let expected_states = vec(ACTIVE);
        include CheckProposalStates<TokenT, ActionT> { expected_states };

        let sender = signer::address_of(signer);
        aborts_if !exists<Vote<TokenT>>(sender);
        let vote = global<Vote<TokenT>>(sender);
        include CheckVoteOnProposal<TokenT> { vote, proposer_address, proposal_id };
        include vote.agree != agree ==> CheckChangeVote<TokenT, ActionT> { vote, proposer_address };

        ensures vote.agree != agree ==> vote.agree == agree;
    }

    spec schema CheckFlipVote<TokenT, ActionT> {
        my_vote: Vote<TokenT>;
        proposal: Proposal<TokenT, ActionT>;
        aborts_if my_vote.agree && proposal.for_votes < my_vote.stake.value;
        aborts_if my_vote.agree && proposal.against_votes + my_vote.stake.value > MAX_U128;
        aborts_if !my_vote.agree && proposal.against_votes < my_vote.stake.value;
        aborts_if !my_vote.agree && proposal.for_votes + my_vote.stake.value > MAX_U128;
    }

    spec do_flip_vote {
        include CheckFlipVote<TokenT, ActionT>;
        ensures my_vote.agree == !old(my_vote).agree;
    }


    spec revoke_vote {
        pragma verify = false;
        include AbortIfDaoInfoNotExist<TokenT>;
        let expected_states = vec(ACTIVE);
        include CheckProposalStates<TokenT, ActionT> { expected_states };
        let sender = signer::address_of(signer);

        aborts_if !exists<Vote<TokenT>>(sender);
        let vote = global<Vote<TokenT>>(sender);
        include CheckVoteOnProposal<TokenT> { vote, proposer_address, proposal_id };
        include CheckRevokeVote<TokenT, ActionT> {
            vote,
            proposal: global<Proposal<TokenT, ActionT>>(proposer_address),
            to_revoke: voting_power,
        };

        modifies global<Vote<TokenT>>(sender);
        modifies global<Proposal<TokenT, ActionT>>(proposer_address);
        modifies global<DaoGlobalInfo<TokenT>>(@0x2);

        ensures global<Vote<TokenT>>(sender).stake.value + result.value == old(
            global<Vote<TokenT>>(sender)
        ).stake.value;
        ensures result.value == voting_power;
    }

    spec schema CheckRevokeVote<TokenT, ActionT> {
        vote: Vote<TokenT>;
        proposal: Proposal<TokenT, ActionT>;
        to_revoke: u128;
        aborts_if vote.stake.value < to_revoke;
        aborts_if vote.agree && proposal.for_votes < to_revoke;
        aborts_if !vote.agree && proposal.against_votes < to_revoke;
    }

    spec do_revoke_vote {
        include CheckRevokeVote<TokenT, ActionT>;
        ensures vote.agree ==> old(proposal).for_votes == proposal.for_votes + to_revoke;
        ensures !vote.agree ==> old(proposal).against_votes == proposal.against_votes + to_revoke;
        ensures result.value == to_revoke;
    }

    spec unstake_votes {
        pragma verify = false;
        let expected_states = vec(DEFEATED);
        let expected_states1 = concat(expected_states, vec(AGREED));
        let expected_states2 = concat(expected_states1, vec(QUEUED));
        let expected_states3 = concat(expected_states2, vec(EXECUTABLE));
        let expected_states4 = concat(expected_states3, vec(EXTRACTED));
        aborts_if expected_states4[0] != DEFEATED;
        aborts_if expected_states4[1] != AGREED;
        aborts_if expected_states4[2] != QUEUED;
        aborts_if expected_states4[3] != EXECUTABLE;
        aborts_if expected_states4[4] != EXTRACTED;
        include spec_proposal_exists<TokenT, ActionT>(proposer_address, proposal_id) ==>
        CheckProposalStates<TokenT, ActionT> { expected_states: expected_states4 };
        let sender = signer::address_of(signer);
        aborts_if !exists<Vote<TokenT>>(sender);
        let vote = global<Vote<TokenT>>(sender);
        include CheckVoteOnProposal<TokenT> { vote, proposer_address, proposal_id };
        ensures !exists<Vote<TokenT>>(sender);
        ensures result.value == old(vote).stake.value;
    }


    spec queue_proposal_action {
        pragma verify = false;
        let expected_states = vec(AGREED);
        include CheckProposalStates<TokenT, ActionT> { expected_states };

        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if timestamp::spec_now_microseconds() + proposal.action_delay > MAX_U64;
        ensures proposal.eta >= timestamp::spec_now_milliseconds();
    }


    spec extract_proposal_action {
        pragma aborts_if_is_partial = false;
        let expected_states = vec(EXECUTABLE);
        include CheckProposalStates<TokenT, ActionT> { expected_states };
        modifies global<Proposal<TokenT, ActionT>>(proposer_address);
        ensures option::is_none(global<Proposal<TokenT, ActionT>>(proposer_address).action);
    }


    spec destroy_terminated_proposal {
        let expected_states = concat(vec(DEFEATED), vec(EXTRACTED));
        aborts_if len(expected_states) != 2;
        aborts_if expected_states[0] != DEFEATED;
        aborts_if expected_states[1] != EXTRACTED;

        aborts_if !exists<Proposal<TokenT, ActionT>>(proposer_address);
        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if proposal.id != proposal_id;
        include AbortIfTimestampNotExist;
        let current_time = timestamp::spec_now_milliseconds();
        let state = do_proposal_state(proposal, current_time);
        aborts_if (forall s in expected_states: s != state);
        aborts_if state == DEFEATED && option::is_none(global<Proposal<TokenT, ActionT>>(proposer_address).action);
        aborts_if state == EXTRACTED && option::is_some(global<Proposal<TokenT, ActionT>>(proposer_address).action);
        modifies global<Proposal<TokenT, ActionT>>(proposer_address);
    }

    spec proposal_exists {
        ensures exists<Proposal<TokenT, ActionT>>(proposer_address) &&
            borrow_global<Proposal<TokenT, ActionT>>(proposer_address).id == proposal_id ==>
            result;
    }

    spec fun spec_proposal_exists<TokenT, ActionT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64,
    ): bool {
        if (exists<Proposal<TokenT, ActionT>>(proposer_address)) {
            let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
            proposal.id == proposal_id
        } else {
            false
        }
    }

    spec schema CheckProposalStates<TokenT, ActionT> {
        proposer_address: address;
        proposal_id: u64;
        expected_states: vector<u8>;
        aborts_if !exists<Proposal<TokenT, ActionT>>(proposer_address);

        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if proposal.id != proposal_id;

        include AbortIfTimestampNotExist;
        let current_time = timestamp::spec_now_milliseconds();
        let state = do_proposal_state(proposal, current_time);
        aborts_if (forall s in expected_states: s != state);
    }

    spec proposal_state {
        use starcoin_framework::system_addresses;

        include AbortIfTimestampNotExist;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if !exists<Proposal<TokenT, ActionT>>(proposer_address);

        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if proposal.id != proposal_id;
    }

    spec proposal_info {
        aborts_if !exists<Proposal<TokenT, ActionT>>(proposer_address);
    }

    spec vote_of {
        aborts_if !exists<Vote<TokenT>>(voter);
        let vote = global<Vote<TokenT>>(voter);
        include CheckVoteOnProposal<TokenT> { vote, proposer_address, proposal_id };
    }

    spec generate_next_proposal_id {
        include GenerateNextProposalIdSchema<TokenT>;
        ensures result == old(global<DaoGlobalInfo<TokenT>>(@0x2).next_proposal_id);
    }

    spec schema GenerateNextProposalIdSchema<TokenT> {
        aborts_if global<DaoGlobalInfo<TokenT>>(@0x2).next_proposal_id >= MAX_U64;
        modifies global<DaoGlobalInfo<TokenT>>(@0x2);
        ensures
            global<DaoGlobalInfo<TokenT>>(@0x2).next_proposal_id ==
                old(global<DaoGlobalInfo<TokenT>>(@0x2).next_proposal_id) + 1;
    }


    spec voting_delay {
        aborts_if false;
    }

    spec voting_period {
        aborts_if false;
    }

    spec schema CheckQuorumVotes<TokenT> {
        aborts_if false;
        // aborts_if option::destroy_some(coin::supply<TokenT>()) * spec_dao_config<TokenT>(
        // ).voting_quorum_rate > MAX_U128;
    }

    spec quorum_votes {
        pragma verify = false;
        // include CheckQuorumVotes<TokenT>;
    }

    spec fun spec_quorum_votes<TokenT>(): u128 {
        // let supply = option::destroy_some(coin::supply<TokenT>()) - treasury::spec_balance<TokenT>();
        // supply * spec_dao_config<TokenT>().voting_quorum_rate / 100
        0
    }

    spec voting_quorum_rate {
        aborts_if false;
        ensures result == global<on_chain_config::Config<DaoConfig<TokenT>>>(
            (@0x2)
        ).payload.voting_quorum_rate;
    }


    spec min_action_delay {
        aborts_if false;
        ensures result == spec_dao_config<TokenT>().min_action_delay;
    }

    spec get_config {
        aborts_if false;
        ensures result == global<on_chain_config::Config<DaoConfig<TokenT>>>(@0x2).payload;
    }


    spec fun spec_dao_config<TokenT>(): DaoConfig<TokenT> {
        global<on_chain_config::Config<DaoConfig<TokenT>>>((@0x2)).payload
    }


    spec schema CheckModifyConfigWithCap<TokenT> {
        cap: on_chain_config::ModifyConfigCapability<DaoConfig<TokenT>>;
        aborts_if cap.account_address != @0x2;
        aborts_if !exists<on_chain_config::Config<DaoConfig<TokenT>>>(cap.account_address);
    }


    spec modify_dao_config {
        include CheckModifyConfigWithCap<TokenT>;
        aborts_if voting_quorum_rate > 0 && voting_quorum_rate > 100;
    }


    spec set_voting_delay {
        include CheckModifyConfigWithCap<TokenT>;
        aborts_if value == 0;
    }

    spec set_voting_period {
        include CheckModifyConfigWithCap<TokenT>;
        aborts_if value == 0;
    }


    spec set_voting_quorum_rate {
        aborts_if !(value > 0 && value <= 100);
        include CheckModifyConfigWithCap<TokenT>;
    }

    spec set_min_action_delay {
        aborts_if value == 0;
        include CheckModifyConfigWithCap<TokenT>;
    }
}
