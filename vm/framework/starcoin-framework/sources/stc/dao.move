module starcoin_framework::dao {

    use std::error;
    use std::option;
    use std::signer;
    use starcoin_framework::treasury;

    use starcoin_framework::stc_util;
    use starcoin_framework::timestamp;
    use starcoin_framework::on_chain_config;
    use starcoin_framework::account;
    use starcoin_framework::coin;
    use starcoin_framework::event;

    /// Proposal state
    const PENDING: u8 = 1;
    const ACTIVE: u8 = 2;
    const DEFEATED: u8 = 3;
    const AGREED: u8 = 4;
    const QUEUED: u8 = 5;
    const EXECUTABLE: u8 = 6;
    const EXTRACTED: u8 = 7;

    /// global DAO info of the specified token type `Token`.
    struct DaoGlobalInfo<phantom Token: store> has key {
        /// next proposal id.
        next_proposal_id: u64,
        /// proposal creating event.
        proposal_create_event: event::EventHandle<ProposalCreatedEvent>,
        /// voting event.
        vote_changed_event: event::EventHandle<VoteChangedEvent>,
    }

    /// Configuration of the `Token`'s DAO.
    struct DaoConfig<phantom TokenT: copy + drop + store> has copy, drop, store {
        /// after proposal created, how long use should wait before he can vote (in milliseconds)
        voting_delay: u64,
        /// how long the voting window is (in milliseconds).
        voting_period: u64,
        /// the quorum rate to agree on the proposal.
        /// if 50% votes needed, then the voting_quorum_rate should be 50.
        /// it should between (0, 100].
        voting_quorum_rate: u8,
        /// how long the proposal should wait before it can be executed (in milliseconds).
        min_action_delay: u64,
    }

    /// emitted when proposal created.
    struct ProposalCreatedEvent has drop, store {
        /// the proposal id.
        proposal_id: u64,
        /// proposer is the user who create the proposal.
        proposer: address,
    }

    /// emitted when user vote/revoke_vote.
    struct VoteChangedEvent has drop, store {
        /// the proposal id.
        proposal_id: u64,
        /// the voter.
        voter: address,
        /// creator of the proposal.
        proposer: address,
        /// agree with the proposal or not
        agree: bool,
        /// latest vote count of the voter.
        vote: u128,
    }

    /// Proposal data struct.
    struct Proposal<phantom Token: store, Action: store> has key {
        /// id of the proposal
        id: u64,
        /// creator of the proposal
        proposer: address,
        /// when voting begins.
        start_time: u64,
        /// when voting ends.
        end_time: u64,
        /// count of voters who agree with the proposal
        for_votes: u128,
        /// count of voters who're against the proposal
        against_votes: u128,
        /// executable after this time.
        eta: u64,
        /// after how long, the agreed proposal can be executed.
        action_delay: u64,
        /// how many votes to reach to make the proposal pass.
        quorum_votes: u128,
        /// proposal action.
        action: option::Option<Action>,
    }

    /// User vote info.
    struct Vote<phantom TokenT: store> has key {
        /// vote for the proposal under the `proposer`.
        proposer: address,
        /// proposal id.
        id: u64,
        /// how many tokens to stake.
        stake: coin::Coin<TokenT>,
        /// vote for or vote against.
        agree: bool,
    }

    const ERR_NOT_AUTHORIZED: u64 = 1401;
    const ERR_ACTION_DELAY_TOO_SMALL: u64 = 1402;
    const ERR_PROPOSAL_STATE_INVALID: u64 = 1403;
    const ERR_PROPOSAL_ID_MISMATCH: u64 = 1404;
    const ERR_PROPOSER_MISMATCH: u64 = 1405;
    const ERR_QUORUM_RATE_INVALID: u64 = 1406;
    const ERR_CONFIG_PARAM_INVALID: u64 = 1407;
    const ERR_VOTE_STATE_MISMATCH: u64 = 1408;
    const ERR_ACTION_MUST_EXIST: u64 = 1409;
    const ERR_VOTED_OTHERS_ALREADY: u64 = 1410;

    /// plugin function, can only be called by token issuer.
    /// Any token who wants to have gov functionality
    /// can optin this module by call this `register function`.
    public fun plugin<TokenT: copy + drop + store>(
        signer: &signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ) {
        let token_issuer = stc_util::token_issuer<TokenT>();
        assert!(signer::address_of(signer) == token_issuer, error::not_found(ERR_NOT_AUTHORIZED));
        // let proposal_id = ProposalId {next: 0};
        let gov_info = DaoGlobalInfo<TokenT> {
            next_proposal_id: 0,
            proposal_create_event: account::new_event_handle<ProposalCreatedEvent>(signer),
            vote_changed_event: account::new_event_handle<VoteChangedEvent>(signer),
        };
        move_to(signer, gov_info);
        let config = new_dao_config<TokenT>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        );
        on_chain_config::publish_new_config(signer, config);
    }


    /// create a dao config
    public fun new_dao_config<TokenT: copy + drop + store>(
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ): DaoConfig<TokenT> {
        assert!(voting_delay > 0, error::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        assert!(voting_period > 0, error::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        assert!(
            voting_quorum_rate > 0 && voting_quorum_rate <= 100,
            error::invalid_argument(ERR_CONFIG_PARAM_INVALID),
        );
        assert!(min_action_delay > 0, error::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        DaoConfig { voting_delay, voting_period, voting_quorum_rate, min_action_delay }
    }


    /// propose a proposal.
    /// `action`: the actual action to execute.
    /// `action_delay`: the delay to execute after the proposal is agreed
    public fun propose<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        signer: &signer,
        action: ActionT,
        action_delay: u64,
    ) acquires DaoGlobalInfo {
        if (action_delay == 0) {
            action_delay = min_action_delay<TokenT>();
        } else {
            assert!(action_delay >= min_action_delay<TokenT>(), error::invalid_argument(ERR_ACTION_DELAY_TOO_SMALL));
        };
        let proposal_id = generate_next_proposal_id<TokenT>();
        let proposer = signer::address_of(signer);
        let start_time = timestamp::now_milliseconds() + voting_delay<TokenT>();
        let quorum_votes = quorum_votes<TokenT>();
        let proposal = Proposal<TokenT, ActionT> {
            id: proposal_id,
            proposer,
            start_time,
            end_time: start_time + voting_period<TokenT>(),
            for_votes: 0,
            against_votes: 0,
            eta: 0,
            action_delay,
            quorum_votes,
            action: option::some(action),
        };
        move_to(signer, proposal);
        // emit event
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(stc_util::token_issuer<TokenT>());
        event::emit_event(
            &mut gov_info.proposal_create_event,
            ProposalCreatedEvent { proposal_id, proposer },
        );
    }


    /// votes for a proposal.
    /// User can only vote once, then the stake is locked,
    /// which can only be unstaked by user after the proposal is expired, or cancelled, or executed.
    /// So think twice before casting vote.
    public fun cast_vote<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        stake: coin::Coin<TokenT>,
        agree: bool,
    ) acquires Proposal, DaoGlobalInfo, Vote {
        {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // only when proposal is active, use can cast vote.
            assert!(state == ACTIVE, error::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        assert!(proposal.id == proposal_id, error::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        let sender = signer::address_of(signer);
        let total_voted = if (exists<Vote<TokenT>>(sender)) {
            let my_vote = borrow_global_mut<Vote<TokenT>>(sender);
            assert!(my_vote.id == proposal_id, error::invalid_argument(ERR_VOTED_OTHERS_ALREADY));
            assert!(my_vote.agree == agree, error::invalid_state(ERR_VOTE_STATE_MISMATCH));

            do_cast_vote(proposal, my_vote, stake);
            coin::value(&my_vote.stake)
        } else {
            let my_vote = Vote<TokenT> {
                proposer: proposer_address,
                id: proposal_id,
                stake: coin::zero(),
                agree,
            };
            do_cast_vote(proposal, &mut my_vote, stake);
            let total_voted = coin::value(&my_vote.stake);
            move_to(signer, my_vote);
            total_voted
        };

        // emit event
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(stc_util::token_issuer<TokenT>());
        event::emit_event(
            &mut gov_info.vote_changed_event,
            VoteChangedEvent {
                proposal_id,
                proposer: proposer_address,
                voter: sender,
                agree,
                vote: (total_voted as u128),
            },
        );
    }


    fun do_cast_vote<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposal: &mut Proposal<TokenT, ActionT>,
        vote: &mut Vote<TokenT>,
        stake: coin::Coin<TokenT>
    ) {
        let stake_value = coin::value(&stake);
        coin::merge(&mut vote.stake, stake);
        if (vote.agree) {
            proposal.for_votes = proposal.for_votes + (stake_value as u128);
        } else {
            proposal.against_votes = proposal.against_votes + (stake_value as u128);
        };
    }


    /// Let user change their vote during the voting time.
    public fun change_vote<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
    ) acquires Proposal, DaoGlobalInfo, Vote {
        {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // only when proposal is active, user can change vote.
            assert!(state == ACTIVE, error::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        assert!(proposal.id == proposal_id, error::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        let my_vote = borrow_global_mut<Vote<TokenT>>(signer::address_of(signer));
        {
            assert!(my_vote.proposer == proposer_address, error::invalid_argument(ERR_PROPOSER_MISMATCH));
            assert!(my_vote.id == proposal_id, error::invalid_argument(ERR_VOTED_OTHERS_ALREADY));
        };

        // flip the vote
        if (my_vote.agree != agree) {
            let total_voted = do_flip_vote(my_vote, proposal);
            // emit event
            let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(stc_util::token_issuer<TokenT>());
            event::emit_event(
                &mut gov_info.vote_changed_event,
                VoteChangedEvent {
                    proposal_id,
                    proposer: proposer_address,
                    voter: signer::address_of(signer),
                    agree,
                    vote: total_voted,
                },
            );
        };
    }


    fun do_flip_vote<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        my_vote: &mut Vote<TokenT>,
        proposal: &mut Proposal<TokenT, ActionT>
    ): u128 {
        my_vote.agree = !my_vote.agree;
        let total_voted = (coin::value(&my_vote.stake) as u128);
        if (my_vote.agree) {
            proposal.for_votes = proposal.for_votes + total_voted;
            proposal.against_votes = proposal.against_votes - total_voted;
        } else {
            proposal.for_votes = proposal.for_votes - total_voted;
            proposal.against_votes = proposal.against_votes + total_voted;
        };
        total_voted
    }


    /// Revoke some voting powers from vote on `proposal_id` of `proposer_address`.
    public fun revoke_vote<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        voting_power: u128,
    ): coin::Coin<TokenT> acquires Proposal, Vote, DaoGlobalInfo {
        {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // only when proposal is active, user can revoke vote.
            assert!(state == ACTIVE, error::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        // get proposal
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);

        // get vote
        let my_vote = move_from<Vote<TokenT>>(signer::address_of(signer));
        {
            assert!(my_vote.proposer == proposer_address, error::invalid_argument(ERR_PROPOSER_MISMATCH));
            assert!(my_vote.id == proposal_id, error::invalid_argument(ERR_VOTED_OTHERS_ALREADY));
        };
        // revoke vote on proposal
        let reverted_stake = do_revoke_vote(proposal, &mut my_vote, voting_power);
        // emit vote changed event
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(stc_util::token_issuer<TokenT>());
        event::emit_event(
            &mut gov_info.vote_changed_event,
            VoteChangedEvent {
                proposal_id,
                proposer: proposer_address,
                voter: signer::address_of(signer),
                agree: my_vote.agree,
                vote: (coin::value(&my_vote.stake) as u128),
            },
        );

        // if user has no stake, destroy his vote. resolve https://github.com/starcoinorg/starcoin/issues/2925.
        if (coin::value(&my_vote.stake) == 0) {
            let Vote { stake, proposer: _, id: _, agree: _ } = my_vote;
            coin::destroy_zero(stake);
        } else {
            move_to(signer, my_vote);
        };

        reverted_stake
    }

    fun do_revoke_vote<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposal: &mut Proposal<TokenT, ActionT>,
        vote: &mut Vote<TokenT>,
        to_revoke: u128
    ): coin::Coin<TokenT> {
        spec {
            assume vote.stake.value >= to_revoke;
        };
        let reverted_stake = coin::extract(&mut vote.stake, (to_revoke as u64));
        if (vote.agree) {
            proposal.for_votes = proposal.for_votes - to_revoke;
        } else {
            proposal.against_votes = proposal.against_votes - to_revoke;
        };
        spec {
            assert coin::value(reverted_stake) == to_revoke;
        };
        reverted_stake
    }

    /// Retrieve back my staked token voted for a proposal.
    public fun unstake_votes<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ): coin::Coin<TokenT> acquires Proposal, Vote {
        // only check state when proposal exists.
        // because proposal can be destroyed after it ends in DEFEATED or EXTRACTED state.
        if (proposal_exists<TokenT, ActionT>(proposer_address, proposal_id)) {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // Only after vote period end, user can unstake his votes.
            assert!(state > ACTIVE, error::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        let Vote { proposer, id, stake, agree: _ } = move_from<Vote<TokenT>>(
            signer::address_of(signer),
        );
        // these checks are still required.
        assert!(proposer == proposer_address, error::not_found(ERR_PROPOSER_MISMATCH));
        assert!(id == proposal_id, error::invalid_argument(ERR_VOTED_OTHERS_ALREADY));
        stake
    }


    /// queue agreed proposal to execute.
    public entry fun queue_proposal_action<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires Proposal {
        // Only agreed proposal can be submitted.
        assert!(
            proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == AGREED,
            error::invalid_state(ERR_PROPOSAL_STATE_INVALID)
        );
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        proposal.eta = timestamp::now_milliseconds() + proposal.action_delay;
    }


    /// extract proposal action to execute.
    public fun extract_proposal_action<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64,
    ): ActionT acquires Proposal {
        // Only executable proposal's action can be extracted.
        assert!(
            proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == EXECUTABLE,
            error::invalid_state(ERR_PROPOSAL_STATE_INVALID),
        );
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        let action: ActionT = option::extract(&mut proposal.action);
        action
    }


    /// remove terminated proposal from proposer
    public entry fun destroy_terminated_proposal<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires Proposal {
        let proposal_state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
        assert!(
            proposal_state == DEFEATED || proposal_state == EXTRACTED,
            error::invalid_state(ERR_PROPOSAL_STATE_INVALID),
        );
        let Proposal {
            id: _,
            proposer: _,
            start_time: _,
            end_time: _,
            for_votes: _,
            against_votes: _,
            eta: _,
            action_delay: _,
            quorum_votes: _,
            action,
        } = move_from<Proposal<TokenT, ActionT>>(proposer_address);
        if (proposal_state == DEFEATED) {
            let _ = option::extract(&mut action);
        };
        option::destroy_none(action);
    }

    /// check whether a proposal exists in `proposer_address` with id `proposal_id`.
    public fun proposal_exists<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64,
    ): bool acquires Proposal {
        if (exists<Proposal<TokenT, ActionT>>(proposer_address)) {
            let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
            return proposal.id == proposal_id
        };
        false
    }

    /// Get the proposal state.
    public fun proposal_state<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64,
    ): u8 acquires Proposal {
        let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
        assert!(proposal.id == proposal_id, error::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        let current_time = timestamp::now_milliseconds();
        do_proposal_state(proposal, current_time)
    }

    fun do_proposal_state<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposal: &Proposal<TokenT, ActionT>,
        current_time: u64,
    ): u8 {
        if (current_time < proposal.start_time) {
            // Pending
            PENDING
        } else if (current_time <= proposal.end_time) {
            // Active
            ACTIVE
        } else if (proposal.for_votes <= proposal.against_votes ||
            proposal.for_votes < proposal.quorum_votes) {
            // Defeated
            DEFEATED
        } else if (proposal.eta == 0) {
            // Agreed.
            AGREED
        } else if (current_time < proposal.eta) {
            // Queued, waiting to execute
            QUEUED
        } else if (option::is_some(&proposal.action)) {
            EXECUTABLE
        } else {
            EXTRACTED
        }
    }


    /// get proposal's information.
    /// return: (id, start_time, end_time, for_votes, against_votes).
    public fun proposal_info<TokenT: copy + drop + store, ActionT: copy + drop + store>(
        proposer_address: address,
    ): (u64, u64, u64, u128, u128) acquires Proposal {
        let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
        (proposal.id, proposal.start_time, proposal.end_time, proposal.for_votes, proposal.against_votes)
    }

    /// Get voter's vote info on proposal with `proposal_id` of `proposer_address`.
    public fun vote_of<TokenT: copy + drop + store>(
        voter: address,
        proposer_address: address,
        proposal_id: u64,
    ): (bool, u128) acquires Vote {
        let vote = borrow_global<Vote<TokenT>>(voter);
        assert!(vote.proposer == proposer_address, error::not_found(ERR_PROPOSER_MISMATCH));
        assert!(vote.id == proposal_id, error::invalid_argument(ERR_VOTED_OTHERS_ALREADY));
        (vote.agree, (coin::value(&vote.stake) as u128))
    }


    /// Check whether voter has voted on proposal with `proposal_id` of `proposer_address`.
    public fun has_vote<TokenT: copy + drop + store>(
        voter: address,
        proposer_address: address,
        proposal_id: u64,
    ): bool acquires Vote {
        if (!exists<Vote<TokenT>>(voter)) {
            return false
        };

        let vote = borrow_global<Vote<TokenT>>(voter);
        vote.proposer == proposer_address && vote.id == proposal_id
    }

    fun generate_next_proposal_id<TokenT: store>(): u64 acquires DaoGlobalInfo {
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(stc_util::token_issuer<TokenT>());
        let proposal_id = gov_info.next_proposal_id;
        gov_info.next_proposal_id = proposal_id + 1;
        proposal_id
    }



    //// Helper functions

    //// Query functions

    /// get default voting delay of the DAO.
    public fun voting_delay<TokenT: copy + drop + store>(): u64 {
        get_config<TokenT>().voting_delay
    }

    /// get the default voting period of the DAO.
    public fun voting_period<TokenT: copy + drop + store>(): u64 {
        get_config<TokenT>().voting_period
    }


    /// Quorum votes to make proposal pass.
    public fun quorum_votes<TokenT: copy + drop + store>(): u128 {
        let market_cap = option::destroy_some(coin::supply<TokenT>());
        let balance_in_treasury = treasury::balance<TokenT>();
        let supply = market_cap - balance_in_treasury;
        let rate = voting_quorum_rate<TokenT>();
        let rate = (rate as u128);
        supply * rate / 100
    }

    /// Get the quorum rate in percent.
    public fun voting_quorum_rate<TokenT: copy + drop + store>(): u8 {
        get_config<TokenT>().voting_quorum_rate
    }

    /// Get the min_action_delay of the DAO.
    public fun min_action_delay<TokenT: copy + drop + store>(): u64 {
        get_config<TokenT>().min_action_delay
    }

    fun get_config<TokenT: copy + drop + store>(): DaoConfig<TokenT> {
        let token_issuer = stc_util::token_issuer<TokenT>();
        on_chain_config::get_by_address<DaoConfig<TokenT>>(token_issuer)
    }


    /// update function, modify dao config.
    /// if any param is 0, it means no change to that param.
    public fun modify_dao_config<TokenT: copy + drop + store>(
        cap: &mut on_chain_config::ModifyConfigCapability<DaoConfig<TokenT>>,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ) {
        assert!(
            on_chain_config::account_address(cap) == stc_util::token_issuer<TokenT>(),
            error::invalid_argument(ERR_NOT_AUTHORIZED)
        );
        let config = get_config<TokenT>();
        if (voting_period > 0) {
            config.voting_period = voting_period;
        };
        if (voting_delay > 0) {
            config.voting_delay = voting_delay;
        };
        if (voting_quorum_rate > 0) {
            assert!(voting_quorum_rate <= 100, error::invalid_argument(ERR_QUORUM_RATE_INVALID));
            config.voting_quorum_rate = voting_quorum_rate;
        };
        if (min_action_delay > 0) {
            config.min_action_delay = min_action_delay;
        };
        on_chain_config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }


    /// set voting delay
    public fun set_voting_delay<TokenT: copy + drop + store>(
        cap: &mut on_chain_config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert!(
            on_chain_config::account_address(cap) == stc_util::token_issuer<TokenT>(),
            error::invalid_argument(ERR_NOT_AUTHORIZED)
        );
        assert!(value > 0, error::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        let config = get_config<TokenT>();
        config.voting_delay = value;
        on_chain_config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }


    /// set voting period
    public fun set_voting_period<TokenT: copy + drop + store>(
        cap: &mut on_chain_config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert!(
            on_chain_config::account_address(cap) == stc_util::token_issuer<TokenT>(),
            error::invalid_argument(ERR_NOT_AUTHORIZED)
        );
        assert!(value > 0, error::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        let config = get_config<TokenT>();
        config.voting_period = value;
        on_chain_config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    /// set voting quorum rate
    public fun set_voting_quorum_rate<TokenT: copy + drop + store>(
        cap: &mut on_chain_config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u8,
    ) {
        assert!(
            on_chain_config::account_address(cap) == stc_util::token_issuer<TokenT>(),
            error::invalid_argument(ERR_NOT_AUTHORIZED)
        );
        assert!(value <= 100 && value > 0, error::invalid_argument(ERR_QUORUM_RATE_INVALID));
        let config = get_config<TokenT>();
        config.voting_quorum_rate = value;
        on_chain_config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    /// set min action delay
    public fun set_min_action_delay<TokenT: copy + drop + store>(
        cap: &mut on_chain_config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert!(
            on_chain_config::account_address(cap) == stc_util::token_issuer<TokenT>(),
            error::invalid_argument(ERR_NOT_AUTHORIZED)
        );
        assert!(value > 0, error::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        let config = get_config<TokenT>();
        config.min_action_delay = value;
        on_chain_config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }
}
