address 0x1 {
module Dao {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Timestamp;
    use 0x1::Option;
    use 0x1::Config;
    use 0x1::Event;
    use 0x1::Errors;
    // use 0x1::Math;

    spec module {
        pragma verify;
        pragma aborts_if_is_partial;
        pragma aborts_if_is_strict = false;
    }

    /// default voting_delay: 1hour
    const DEFAULT_VOTING_DELAY: u64 = 60 * 60;
    /// default voting_period: 2days
    const DEFAULT_VOTING_PERIOD: u64 = 60 * 60 * 24 * 2;
    /// default quorum rate: 4% of toal token supply.
    const DEFAULT_VOTEING_QUORUM_RATE: u8 = 4;
    /// default action_delay: 1days
    const DEFAULT_MIN_ACTION_DELAY: u64 = 60 * 60 * 24;

    /// default min_action_delay
    public fun default_min_action_delay(): u64 {
        DEFAULT_MIN_ACTION_DELAY
    }
    spec fun default_min_action_delay {
        aborts_if false;
    }

    public fun default_voting_delay(): u64 {
        DEFAULT_VOTING_DELAY
    }
    spec fun default_voting_delay {
        aborts_if false;
    }

    public fun default_voting_period(): u64 {
        DEFAULT_VOTING_PERIOD
    }
    spec fun default_voting_period {
        aborts_if false;
    }

    public fun default_voting_quorum_rate(): u8 {
        DEFAULT_VOTEING_QUORUM_RATE
    }
    spec fun default_voting_quorum_rate {
        aborts_if false;
    }

    /// Proposal state
    const PENDING: u8 = 1;
    const ACTIVE: u8 = 2;
    const DEFEATED: u8 = 3;
    const AGREED: u8 = 4;
    const QUEUED: u8 = 5;
    const EXECUTABLE: u8 = 6;
    const EXTRACTED: u8 = 7;

    resource struct DaoGlobalInfo<Token> {
        next_proposal_id: u64,
        proposal_create_event: Event::EventHandle<ProposalCreatedEvent>,
        vote_changed_event: Event::EventHandle<VoteChangedEvent>,
    }

    spec struct DaoGlobalInfo {
        // fix me
        // invariant next_proposal_id <= max_u64();
    }

    struct DaoConfig<TokenT: copyable> {
        /// after proposal created, how long use should wait before he can vote.
        voting_delay: u64,
        /// how long the voting window is.
        voting_period: u64,
        /// the quorum rate to agree on the proposal.
        /// if 50% votes needed, then the voting_quorum_rate should be 50.
        /// it should between (0, 100].
        voting_quorum_rate: u8,
        /// how long the proposal should wait before it can be executed.
        min_action_delay: u64,
    }

    spec struct DaoConfig {
        invariant voting_quorum_rate > 0 && voting_quorum_rate <= 100;
        invariant voting_delay > 0;
        invariant voting_period > 0;
        invariant min_action_delay > 0;
    }

    /// emitted when proposal created.
    struct ProposalCreatedEvent {
        proposal_id: u64,
        proposer: address,
    }

    /// emitted when user vote/revoke_vote.
    struct VoteChangedEvent {
        proposal_id: u64,
        proposer: address,
        voter: address,
        agree: bool,
        /// latest vote of the voter.
        vote: u128,
    }

    /// TODO: support that one can propose mutli proposals.
    resource struct Proposal<Token, Action> {
        id: u64,
        proposer: address,
        start_time: u64,
        end_time: u64,
        for_votes: u128,
        against_votes: u128,
        // executable after this time.
        eta: u64,
        action_delay: u64,
        action: Option::Option<Action>,
    }
    spec struct Proposal {
        // fix me
        // invariant start_time < end_time;
        // invariant action_delay > 0;
    }

    // TODO: allow user do multi votes.
    resource struct Vote<TokenT> {
        proposer: address,
        id: u64,
        stake: Token::Token<TokenT>,
        agree: bool,
    }
    spec struct Vote {
        // fixme
        // invariant stake.value > 0;
    }

    const ERR_NOT_AUTHORIZED: u64 = 1401;
    const ERR_ACTION_DELAY_TOO_SMALL: u64 = 1402;
    const ERR_PROPOSAL_STATE_INVALID: u64 = 1403;
    const ERR_PROPOSAL_ID_MISMATCH: u64 = 1404;
    const ERR_PROPOSER_MISMATCH: u64 = 1405;
    const ERR_QUROM_RATE_INVALID: u64 = 1406;
    const ERR_CONFIG_PARAM_INVALID: u64 = 1407;
    const ERR_VOTE_STATE_MISMATCH: u64 = 1408;

    /// plugin function, can only be called by token issuer.
    /// Any token who wants to has gov functionality
    /// can optin this moudle by call this `register function`.
    public fun plugin<TokenT: copyable>(
        signer: &signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ) {
        // TODO: we can add a token manage cap in Token module.
        // and only token manager can register this.
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        // let proposal_id = ProposalId {next: 0};
        let gov_info = DaoGlobalInfo<TokenT> {
            next_proposal_id: 0,
            proposal_create_event: Event::new_event_handle<ProposalCreatedEvent>(signer),
            vote_changed_event: Event::new_event_handle<VoteChangedEvent>(signer),
        };
        move_to(signer, gov_info);
        let config = new_dao_config<TokenT>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        );
        Config::publish_new_config(signer, config);
    }

    spec fun plugin {
        aborts_if voting_delay == 0;
        aborts_if voting_period == 0;
        aborts_if voting_quorum_rate == 0 || voting_quorum_rate > 100;
        aborts_if min_action_delay == 0;

        let sender = Signer::spec_address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if exists<DaoGlobalInfo<TokenT>>(sender);
        aborts_if exists<Config::Config<DaoConfig<TokenT>>>(sender);
        aborts_if exists<Config::ModifyConfigCapabilityHolder<DaoConfig<TokenT>>>(sender);
    }

    spec schema RequirePluginDao<TokenT: copyable> {
        let token_addr = Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<DaoGlobalInfo<TokenT>>(token_addr);
        aborts_if !exists<Config::Config<DaoConfig<TokenT>>>(token_addr);
    }
    spec schema AbortIfDaoInfoNotExist<TokenT> {
        let token_addr = Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<DaoGlobalInfo<TokenT>>(token_addr);
    }
    spec schema AbortIfDaoConfigNotExist<TokenT> {
        let token_addr = Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<Config::Config<DaoConfig<TokenT>>>(token_addr);
    }
    spec schema AbortIfTimestampNotExist {
        use 0x1::CoreAddresses;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
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
            quorum_votes<TokenT>,

            modify_dao_config<TokenT>,
            set_*<TokenT>;

    }

    /// create a dao config
    public fun new_dao_config<TokenT: copyable>(
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ): DaoConfig<TokenT> {
        assert(voting_delay > 0, Errors::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        assert(voting_period > 0, Errors::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        assert(voting_quorum_rate > 0 && voting_quorum_rate <= 100, Errors::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        assert(min_action_delay > 0, Errors::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        DaoConfig { voting_delay, voting_period, voting_quorum_rate, min_action_delay }
    }

    spec fun new_dao_config {
        aborts_if voting_delay == 0;
        aborts_if voting_period == 0;
        aborts_if voting_quorum_rate == 0 || voting_quorum_rate > 100;
        aborts_if min_action_delay == 0;
    }

    /// propose a proposal.
    /// `action`: the actual action to execute.
    /// `action_delay`: the delay to execute after the proposal is agreed
    public fun propose<TokenT: copyable, ActionT>(
        signer: &signer,
        action: ActionT,
        action_delay: u64,
    ) acquires DaoGlobalInfo {
        if (action_delay == 0) {
            action_delay = min_action_delay<TokenT>();
        } else {
            assert(action_delay >= min_action_delay<TokenT>(), Errors::invalid_argument(ERR_ACTION_DELAY_TOO_SMALL));
        };
        let proposal_id = generate_next_proposal_id<TokenT>();
        let proposer = Signer::address_of(signer);
        let start_time = Timestamp::now_seconds() + voting_delay<TokenT>();
        let proposal = Proposal<TokenT, ActionT> {
            id: proposal_id,
            proposer,
            start_time,
            end_time: start_time + voting_period<TokenT>(),
            for_votes: 0,
            against_votes: 0,
            eta: 0,
            action_delay,
            action: Option::some(action),
        };
        move_to(signer, proposal);
        // emit event
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(Token::token_address<TokenT>());
        Event::emit_event(
            &mut gov_info.proposal_create_event,
            ProposalCreatedEvent { proposal_id, proposer },
        );
    }

    spec fun propose {
        use 0x1::CoreAddresses;
        pragma addition_overflow_unchecked;
        include AbortIfDaoConfigNotExist<TokenT>;
        include AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());

        aborts_if action_delay > 0 && action_delay < spec_dao_config<TokenT>().min_action_delay;

        let sender = Signer::spec_address_of(signer);
        aborts_if exists<Proposal<TokenT, ActionT>>(sender);
        modifies global<DaoGlobalInfo<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());

        ensures exists<Proposal<TokenT, ActionT>>(sender);

        // TODO: figure out the ensures
        // let proposal = global<Proposal<TokenT, ActionT>>(sender);
        // ensures proposal.action_delay > 0;
        // ensures proposal.end_time > proposal.start_time;
    }

    /// votes for a proposal.
    /// User can only vote once, then the stake is locked,
    /// which can only be unstaked by user after the proposal is expired, or cancelled, or executed.
    /// So think twice before casting vote.
    public fun cast_vote<TokenT: copyable, ActionT>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        stake: Token::Token<TokenT>,
        agree: bool,
    ) acquires Proposal, DaoGlobalInfo, Vote {
        {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // only when proposal is active, use can cast vote.
            assert(state == ACTIVE, Errors::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        assert(proposal.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        let sender = Signer::address_of(signer);
        let total_voted = if (exists<Vote<TokenT>>(sender)) {
            let my_vote = borrow_global_mut<Vote<TokenT>>(sender);
            assert(my_vote.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
            assert(my_vote.agree == agree, Errors::invalid_state(ERR_VOTE_STATE_MISMATCH));

            _cast_vote(proposal, my_vote, stake);
            Token::value(&my_vote.stake)
        } else {
            let my_vote = Vote<TokenT> {
                proposer: proposer_address,
                id: proposal_id,
                stake: Token::zero(),
                agree,
            };
            _cast_vote(proposal, &mut my_vote, stake);
            let total_voted = Token::value(&my_vote.stake);
            move_to(signer, my_vote);
            total_voted
        };

        // emit event
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(Token::token_address<TokenT>());
        Event::emit_event(
            &mut gov_info.vote_changed_event,
            VoteChangedEvent {
                proposal_id,
                proposer: proposer_address,
                voter: sender,
                agree,
                vote: total_voted,
            },
        );
    }

    spec schema CheckVoteOnCast<TokenT, ActionT> {

        proposal_id: u64;
        agree: bool;
        voter: address;
        let vote = global<Vote<TokenT>>(voter);
        aborts_if vote.id != proposal_id;
        aborts_if vote.agree != agree;
    }

    spec fun cast_vote {
        include AbortIfDaoConfigNotExist<TokenT>;
        include AbortIfDaoInfoNotExist<TokenT>;
        let expected_states = singleton_vector(ACTIVE);
        include CheckProposalStates<TokenT, ActionT> {expected_states};
        let sender = Signer::spec_address_of(signer);
        let vote_exists = exists<Vote<TokenT>>(sender);
        include vote_exists ==> CheckVoteOnCast<TokenT, ActionT> {
            voter: sender,
            proposal_id: proposal_id,
            agree: agree
        };
        modifies global<Proposal<TokenT, ActionT>>(proposer_address);
        // TODO: figure out why it cannot work.
        // ensures vote_exists ==>
        //     global<Vote<TokenT>>(sender).stake.value == old(global<Vote<TokenT>>(sender)).stake.value + stake.value;
        ensures !vote_exists ==> global<Vote<TokenT>>(sender).stake.value == stake.value;
    }

    fun _cast_vote<TokenT: copyable, ActionT>(proposal: &mut Proposal<TokenT, ActionT>, vote: &mut Vote<TokenT>, stake: Token::Token<TokenT>) {
        let stake_value = Token::value(&stake);
        Token::deposit(&mut vote.stake, stake);
        if (vote.agree) {
            proposal.for_votes = proposal.for_votes + stake_value;
        } else {
            proposal.against_votes = proposal.against_votes + stake_value;
        };
    }

    spec fun _cast_vote {
        ensures vote.stake.value == old(vote).stake.value + stake.value;
        ensures vote.agree ==> old(proposal).for_votes + stake.value == proposal.for_votes;
        ensures vote.agree ==> old(proposal).against_votes == proposal.against_votes;
        ensures !vote.agree ==> old(proposal).against_votes + stake.value == proposal.against_votes;
        ensures !vote.agree ==> old(proposal).for_votes == proposal.for_votes;
    }


    /// Let user change their vote during the voting time.
    public fun change_vote<TokenT: copyable, ActionT>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
    ) acquires Proposal, DaoGlobalInfo, Vote {
        {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // only when proposal is active, use can change vote.
            assert(state == ACTIVE, Errors::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        assert(proposal.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        let my_vote = borrow_global_mut<Vote<TokenT>>(Signer::address_of(signer));
        {
            assert(my_vote.proposer == proposer_address, Errors::invalid_argument(ERR_PROPOSER_MISMATCH));
            assert(my_vote.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        };

        // flip the vote
        if (my_vote.agree != agree) {
            let total_voted = _flip_vote(my_vote, proposal);
            // emit event
            let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(Token::token_address<TokenT>());
            Event::emit_event(
                &mut gov_info.vote_changed_event,
                VoteChangedEvent {
                    proposal_id,
                    proposer: proposer_address,
                    voter: Signer::address_of(signer),
                    agree,
                    vote: total_voted,
                },
            );
        };
    }
    spec schema CheckVoteOnProposal<TokenT> {
        vote: Vote<TokenT>;
        proposer_address: address;
        proposal_id: u64;

        aborts_if vote.id != proposal_id;
        aborts_if vote.proposer != proposer_address;
    }

    spec fun change_vote {
        pragma aborts_if_is_partial = true;

        // include AbortIfDaoConfigNotExist<TokenT>;
        let expected_states = singleton_vector(ACTIVE);
        include CheckProposalStates<TokenT, ActionT>{expected_states};

        let sender = Signer::spec_address_of(signer);

        aborts_if !exists<Vote<TokenT>>(sender);
        let vote = global<Vote<TokenT>>(sender);
        include CheckVoteOnProposal<TokenT>{vote, proposer_address, proposal_id};

        include vote.agree != agree ==> AbortIfDaoInfoNotExist<TokenT>;
        ensures vote.agree != agree ==> vote.agree == agree;
    }

    fun _flip_vote<TokenT: copyable, ActionT>(my_vote: &mut Vote<TokenT>, proposal: &mut Proposal<TokenT, ActionT>): u128 {
        my_vote.agree = !my_vote.agree;
        let total_voted = Token::value(&my_vote.stake);
        if (my_vote.agree) {
            proposal.for_votes = proposal.for_votes + total_voted;
            proposal.against_votes = proposal.against_votes - total_voted;
        } else {
            proposal.for_votes = proposal.for_votes - total_voted;
            proposal.against_votes = proposal.against_votes + total_voted;
        };
        total_voted
    }

    spec fun _flip_vote {
        pragma aborts_if_is_partial = false;
        aborts_if my_vote.agree && proposal.for_votes < my_vote.stake.value;
        aborts_if my_vote.agree && proposal.against_votes + my_vote.stake.value > MAX_U128;
        aborts_if !my_vote.agree && proposal.against_votes < my_vote.stake.value;
        aborts_if !my_vote.agree && proposal.for_votes + my_vote.stake.value > MAX_U128;
        ensures my_vote.agree == !old(my_vote).agree;
    }

    /// Revoke some voting powers from vote on `proposal_id` of `proposer_address`.
    public fun revoke_vote<TokenT: copyable, ActionT>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
        voting_power: u128,
    ): Token::Token<TokenT> acquires Proposal, Vote, DaoGlobalInfo {
        {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // only when proposal is active, use can revoke vote.
            assert(state == ACTIVE, Errors::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        // get proposal
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);

        // get vote
        let my_vote = borrow_global_mut<Vote<TokenT>>(Signer::address_of(signer));
        {
            assert(my_vote.proposer == proposer_address, Errors::invalid_argument(ERR_PROPOSER_MISMATCH));
            assert(my_vote.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        };
        // revoke vote on proposal
        let reverted_stake =_revoke_vote(proposal, my_vote, voting_power);
        // emit vote changed event
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(Token::token_address<TokenT>());
        Event::emit_event(
            &mut gov_info.vote_changed_event,
            VoteChangedEvent {
                proposal_id,
                proposer: proposer_address,
                voter: Signer::address_of(signer),
                agree: my_vote.agree,
                vote: Token::value(&my_vote.stake),
            },
        );
        reverted_stake
    }

    spec fun revoke_vote {
        include AbortIfDaoConfigNotExist<TokenT>;
        include AbortIfDaoInfoNotExist<TokenT>;
        let expected_states = singleton_vector(ACTIVE);
        include CheckProposalStates<TokenT, ActionT> {expected_states};
        let sender = Signer::spec_address_of(signer);

        aborts_if !exists<Vote<TokenT>>(sender);
        let vote = global<Vote<TokenT>>(sender);
        include CheckVoteOnProposal<TokenT> {vote, proposer_address, proposal_id};

        modifies global<Vote<TokenT>>(sender);
        modifies global<Proposal<TokenT, ActionT>>(proposer_address);
        modifies global<DaoGlobalInfo<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());

        ensures global<Vote<TokenT>>(sender).stake.value + result.value == old(global<Vote<TokenT>>(sender)).stake.value;
        ensures result.value == voting_power;
    }

    fun _revoke_vote<TokenT: copyable, ActionT>(proposal: &mut Proposal<TokenT, ActionT>, vote: &mut Vote<TokenT>, to_revoke: u128): Token::Token<TokenT> {
        let reverted_stake = Token::withdraw(&mut vote.stake, to_revoke);
        if (vote.agree) {
            proposal.for_votes = proposal.for_votes - to_revoke;
        } else {
            proposal.against_votes = proposal.against_votes - to_revoke;
        };
        spec {
            assert Token::value(reverted_stake) == to_revoke;
        };
        reverted_stake
    }

    spec fun _revoke_vote {
        pragma aborts_if_is_partial = false;
        aborts_if vote.stake.value < to_revoke;
        aborts_if vote.agree && proposal.for_votes < to_revoke;
        aborts_if !vote.agree && proposal.against_votes < to_revoke;
        ensures vote.agree ==> old(proposal).for_votes == proposal.for_votes + to_revoke;
        ensures !vote.agree ==> old(proposal).against_votes == proposal.against_votes + to_revoke;
        ensures result.value == to_revoke;
    }

    /// Retrieve back my staked token voted for a proposal.
    public fun unstake_votes<TokenT: copyable, ActionT>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ): Token::Token<TokenT> acquires Proposal, Vote {
        // only check state when proposal exists.
        // because proposal can be destroyed after it ends in DEFEATED or EXTRACTED state.
        if (proposal_exists<TokenT, ActionT>(proposer_address, proposal_id)) {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // Only after vote period end, user can unstake his votes.
            assert(state > ACTIVE, Errors::invalid_state(ERR_PROPOSAL_STATE_INVALID));
        };
        let Vote { proposer, id, stake, agree: _ } = move_from<Vote<TokenT>>(
            Signer::address_of(signer),
        );
        // these checks are still required.
        assert(proposer == proposer_address, Errors::requires_address(ERR_PROPOSER_MISMATCH));
        assert(id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        stake
    }

    spec fun unstake_votes {
        let expected_states = singleton_vector(DEFEATED);
        let expected_states1 = concat_vector(expected_states,singleton_vector(AGREED));
        let expected_states2 = concat_vector(expected_states1,singleton_vector(QUEUED));
        let expected_states3 = concat_vector(expected_states2,singleton_vector(EXECUTABLE));
        let expected_states4 = concat_vector(expected_states3,singleton_vector(EXTRACTED));
        aborts_if expected_states4[0] != DEFEATED;
        aborts_if expected_states4[1] != AGREED;
        aborts_if expected_states4[2] != QUEUED;
        aborts_if expected_states4[3] != EXECUTABLE;
        aborts_if expected_states4[4] != EXTRACTED;
        include spec_proposal_exists<TokenT, ActionT>(proposer_address, proposal_id) ==>
                    CheckProposalStates<TokenT, ActionT>{expected_states: expected_states4};
        let sender = Signer::address_of(signer);
        aborts_if !exists<Vote<TokenT>>(sender);
        let vote = global<Vote<TokenT>>(sender);
        include CheckVoteOnProposal<TokenT>{vote, proposer_address, proposal_id};
        ensures !exists<Vote<TokenT>>(sender);
        ensures result.value == old(vote).stake.value;
    }


    /// queue agreed proposal to execute.
    public fun queue_proposal_action<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires Proposal {
        // Only agreed proposal can be submitted.
        assert(
            proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == AGREED,
            Errors::invalid_state(ERR_PROPOSAL_STATE_INVALID)
        );
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        proposal.eta = Timestamp::now_seconds() + proposal.action_delay;
    }
    spec fun queue_proposal_action {
        let expected_states = singleton_vector(AGREED);
        include CheckProposalStates<TokenT, ActionT>{expected_states};

        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if Timestamp::spec_now_seconds() + proposal.action_delay > MAX_U64;
        ensures proposal.eta >= Timestamp::spec_now_seconds();
    }

    /// extract proposal action to execute.
    public fun extract_proposal_action<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ): ActionT acquires Proposal {
        // Only executable proposal's action can be extracted.
        assert(
            proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == EXECUTABLE,
            Errors::invalid_state(ERR_PROPOSAL_STATE_INVALID),
        );
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        let action: ActionT = Option::extract(&mut proposal.action);
        action
    }
    spec fun extract_proposal_action {
        let expected_states = singleton_vector(EXECUTABLE);
        include CheckProposalStates<TokenT, ActionT>{expected_states};
        modifies global<Proposal<TokenT, ActionT>>(proposer_address);
        ensures Option::spec_is_none(global<Proposal<TokenT, ActionT>>(proposer_address).action);
    }


    /// remove terminated proposal from proposer
    public fun destroy_terminated_proposal<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires Proposal {
        let proposal_state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
        assert(
            proposal_state == DEFEATED || proposal_state == EXTRACTED,
            Errors::invalid_state(ERR_PROPOSAL_STATE_INVALID),
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
            action,
        } = move_from<Proposal<TokenT, ActionT>>(proposer_address);
        Option::destroy_none(action);
    }

    spec fun destroy_terminated_proposal {
        let expected_states = concat_vector(singleton_vector(DEFEATED), singleton_vector(EXTRACTED));
        aborts_if len(expected_states) != 2;
        aborts_if expected_states[0] != DEFEATED;
        aborts_if expected_states[1] != EXTRACTED;
        include CheckProposalStates<TokenT, ActionT>{expected_states};
        aborts_if Option::spec_is_some(global<Proposal<TokenT, ActionT>>(proposer_address).action);
        ensures !exists<Proposal<TokenT, ActionT>>(proposer_address);
    }

    /// check whether a proposal exists in `proposer_address` with id `proposal_id`.
    public fun proposal_exists<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ): bool acquires Proposal {
        if (exists<Proposal<TokenT, ActionT>>(proposer_address)) {
            let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
            return proposal.id == proposal_id
        };
        false
    }
    spec fun proposal_exists {
        pragma aborts_if_is_partial = false;
        ensures exists<Proposal<TokenT, ActionT>>(proposer_address) &&
                    borrow_global<Proposal<TokenT, ActionT>>(proposer_address).id == proposal_id ==>
                    result;
    }

    spec define spec_proposal_exists<TokenT: copyable, ActionT>(
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

    public fun proposal_state<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ): u8 acquires Proposal {
        let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
        assert(proposal.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        let current_time = Timestamp::now_seconds();
        let quorum_votes = quorum_votes<TokenT>();
        _proposal_state(proposal, current_time, quorum_votes)
    }

    spec schema CheckProposalStates<TokenT, ActionT> {
        proposer_address: address;
        proposal_id: u64;
        expected_states: vector<u8>;
        aborts_if !exists<Proposal<TokenT, ActionT>>(proposer_address);

        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if proposal.id != proposal_id;

        include AbortIfDaoConfigNotExist<TokenT>;
        include AbortIfTimestampNotExist;

        let quorum_votes = spec_quorum_votes<TokenT>();
        let current_time = Timestamp::spec_now_seconds();
        let state = _proposal_state(proposal, current_time, quorum_votes);
        aborts_if (forall s in expected_states : s != state);
    }

    spec fun proposal_state {
        use 0x1::CoreAddresses;
        include AbortIfDaoConfigNotExist<TokenT>;
        include AbortIfTimestampNotExist;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if !exists<Proposal<TokenT, ActionT>>(proposer_address);

        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if proposal.id != proposal_id;
        // TODO: check result
    }

    fun _proposal_state<TokenT: copyable, ActionT>(
        proposal: &Proposal<TokenT, ActionT>,
        current_time: u64,
        quorum_votes: u128,
    ): u8 {
        // let current_time = Timestamp::now_seconds();
        // let quorum_votes = quorum_votes<TokenT>();
        if (current_time < proposal.start_time) {
            // Pending
            PENDING
        } else if (current_time <= proposal.end_time) {
            // Active
            ACTIVE
        } else if (proposal.for_votes <= proposal.against_votes ||
            proposal.for_votes < quorum_votes) {
            // Defeated
            DEFEATED
        } else if (proposal.eta == 0) {
            // Agreed.
            AGREED
        } else if (current_time < proposal.eta) {
            // Queued, waiting to execute
            QUEUED
        } else if (Option::is_some(&proposal.action)) {
            EXECUTABLE
        } else {
            EXTRACTED
        }
    }


    /// get proposal's information.
    /// return: (start_time, end_time, for_votes, against_votes).
    public fun proposal_info<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ): (u64, u64, u128, u128) acquires Proposal {
        let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
        assert(proposal.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        (proposal.start_time, proposal.end_time, proposal.for_votes, proposal.against_votes)
    }

    spec fun proposal_info {
        aborts_if !exists<Proposal<TokenT, ActionT>>(proposer_address);
        let proposal = global<Proposal<TokenT, ActionT>>(proposer_address);
        aborts_if proposal.id != proposal_id;
    }

    /// Get voter's vote info on proposal with `proposal_id` of `proposer_address`.
    public fun vote_of<TokenT: copyable>(
        voter: address,
        proposer_address: address,
        proposal_id: u64,
    ): (bool, u128) acquires Vote {
        let vote = borrow_global<Vote<TokenT>>(voter);
        assert(vote.proposer == proposer_address, Errors::requires_address(ERR_PROPOSER_MISMATCH));
        assert(vote.id == proposal_id, Errors::invalid_argument(ERR_PROPOSAL_ID_MISMATCH));
        (vote.agree, Token::value(&vote.stake))
    }

    spec fun vote_of {
        aborts_if !exists<Vote<TokenT>>(voter);
        let vote = global<Vote<TokenT>>(voter);
        include CheckVoteOnProposal<TokenT>{vote, proposer_address, proposal_id};
    }


    fun generate_next_proposal_id<TokenT>(): u64 acquires DaoGlobalInfo {
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(Token::token_address<TokenT>());
        let proposal_id = gov_info.next_proposal_id;
        gov_info.next_proposal_id = proposal_id + 1;
        proposal_id
    }
    spec fun generate_next_proposal_id  {
        pragma addition_overflow_unchecked;
        modifies global<DaoGlobalInfo<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
        ensures
            global<DaoGlobalInfo<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS()).next_proposal_id ==
            old(global<DaoGlobalInfo<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS()).next_proposal_id) + 1;
        ensures result == old(global<DaoGlobalInfo<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS()).next_proposal_id);
    }

    //// Helper functions

    //// Query functions
    public fun voting_delay<TokenT: copyable>(): u64 {
        get_config<TokenT>().voting_delay
    }

    spec fun voting_delay {
        aborts_if false;
    }

    public fun voting_period<TokenT: copyable>(): u64 {
        get_config<TokenT>().voting_period
    }

    spec fun voting_period {
        aborts_if false;
    }

    /// Quorum votes to make proposal pass.
    public fun quorum_votes<TokenT: copyable>(): u128 {
        let supply = Token::market_cap<TokenT>();
        let rate = voting_quorum_rate<TokenT>();
        // let rate1 = (rate as u64);
        let rate2 = (rate as u128);
        supply * rate2 / 100u128
        // Math::mul_div(supply, (voting_quorum_rate<TokenT>() as u128), 100)
    }
    spec fun quorum_votes {
        // TODO: why
        pragma verify = false;
        // pragma addition_overflow_unchecked;
    }

    spec define spec_quorum_votes<TokenT: copyable>(): u128 {
        let supply = Token::spec_abstract_total_value<TokenT>();
        supply * spec_dao_config<TokenT>().voting_quorum_rate / 100
    }

    public fun voting_quorum_rate<TokenT: copyable>(): u8 {
        get_config<TokenT>().voting_quorum_rate
    }

    spec fun voting_quorum_rate {
        aborts_if false;
        ensures result == global<Config::Config<DaoConfig<TokenT>>>((Token::SPEC_TOKEN_TEST_ADDRESS())).payload.voting_quorum_rate;
    }

    public fun min_action_delay<TokenT: copyable>(): u64 {
        get_config<TokenT>().min_action_delay
    }

    spec fun min_action_delay {
        aborts_if false;
        ensures result == spec_dao_config<TokenT>().min_action_delay;
    }

    fun get_config<TokenT: copyable>(): DaoConfig<TokenT> {
        let token_issuer = Token::token_address<TokenT>();
        Config::get_by_address<DaoConfig<TokenT>>(token_issuer)
    }

    spec fun get_config {
        ensures result == global<Config::Config<DaoConfig<TokenT>>>((Token::SPEC_TOKEN_TEST_ADDRESS())).payload;
    }

    spec module {
        define spec_dao_config<TokenT: copyable>(): DaoConfig<TokenT> {
            global<Config::Config<DaoConfig<TokenT>>>((Token::SPEC_TOKEN_TEST_ADDRESS())).payload
        }
    }

    /// update function
    /// TODO: cap should not be mut to set data.
    public fun modify_dao_config<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ) {
        let config = get_config<TokenT>();
        if (voting_period > 0) {
            config.voting_period = voting_period;
        };
        if (voting_delay > 0) {
            config.voting_delay = voting_delay;
        };
        if (voting_quorum_rate > 0) {
            assert(voting_quorum_rate <= 100, Errors::invalid_argument(ERR_QUROM_RATE_INVALID));
            config.voting_quorum_rate = voting_quorum_rate;
        };
        if (min_action_delay > 0) {
            config.min_action_delay = min_action_delay;
        };
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    public fun set_voting_delay<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert(value > 0, Errors::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        let config = get_config<TokenT>();
        config.voting_delay = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    public fun set_voting_period<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert(value > 0, Errors::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        let config = get_config<TokenT>();
        config.voting_period = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    public fun set_voting_quorum_rate<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u8,
    ) {
        assert(value <= 100 && value > 0, Errors::invalid_argument(ERR_QUROM_RATE_INVALID));
        let config = get_config<TokenT>();
        config.voting_quorum_rate = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    public fun set_min_action_delay<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert(value > 0, Errors::invalid_argument(ERR_CONFIG_PARAM_INVALID));
        let config = get_config<TokenT>();
        config.min_action_delay = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }
}
}