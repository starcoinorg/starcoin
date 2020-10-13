address 0x1 {
module Dao {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Timestamp;
    use 0x1::Option;
    use 0x1::Config;
    use 0x1::Event;

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

    public fun default_voting_delay(): u64 {
        DEFAULT_VOTING_DELAY
    }

    public fun default_voting_period(): u64 {
        DEFAULT_VOTING_PERIOD
    }

    public fun default_voting_quorum_rate(): u8 {
        DEFAULT_VOTEING_QUORUM_RATE
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

    struct DaoConfig<TokenT: copyable> {
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
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

    // TODO: allow user do multi votes.
    resource struct Vote<TokenT> {
        proposer: address,
        id: u64,
        stake: Token::Token<TokenT>,
        agree: bool,
    }

    const ERR_NOT_AUTHORIZED: u64 = 1401;
    const ERR_ACTION_DELAY_TOO_SMALL: u64 = 1402;
    const ERR_PROPOSAL_STATE_INVALID: u64 = 1403;
    const ERR_PROPOSAL_ID_MISMATCH: u64 = 1404;
    const ERR_PROPOSER_MISMATCH: u64 = 1405;
    const ERR_QUROM_RATE_INVALID: u64 = 1406;
    const ERR_CONFIG_PARAM_INVALID: u64 = 1407;

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
        assert(Signer::address_of(signer) == token_issuer, ERR_NOT_AUTHORIZED);
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

    /// create a dao config
    public fun new_dao_config<TokenT: copyable>(
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ): DaoConfig<TokenT> {
        assert(voting_delay > 0, ERR_CONFIG_PARAM_INVALID);
        assert(voting_period > 0, ERR_CONFIG_PARAM_INVALID);
        assert(voting_quorum_rate > 0 && voting_quorum_rate <= 100, ERR_CONFIG_PARAM_INVALID);
        assert(min_action_delay > 0, ERR_CONFIG_PARAM_INVALID);
        DaoConfig { voting_delay, voting_period, voting_quorum_rate, min_action_delay }
    }

    /// propose a proposal.
    /// `action`: the actual action to execute.
    /// `action_delay`: the delay to execute after the proposal is agreed
    public fun propose<TokenT: copyable, ActionT>(
        signer: &signer,
        action: ActionT,
        action_delay: u64,
    ) acquires DaoGlobalInfo {
        assert(action_delay >= min_action_delay<TokenT>(), ERR_ACTION_DELAY_TOO_SMALL);
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
    ) acquires Proposal, DaoGlobalInfo {
        {
            let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
            // only when proposal is active, use can cast vote.
            assert(state == ACTIVE, ERR_PROPOSAL_STATE_INVALID);
        };
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        assert(proposal.id == proposal_id, ERR_PROPOSAL_ID_MISMATCH);
        let stake_value = Token::value(&stake);
        let my_vote = Vote<TokenT> { proposer: proposer_address, id: proposal_id, stake, agree };
        if (agree) {
            proposal.for_votes = proposal.for_votes + stake_value;
        } else {
            proposal.against_votes = proposal.against_votes + stake_value;
        };
        move_to(signer, my_vote);
        // emit event
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(Token::token_address<TokenT>());
        Event::emit_event(
            &mut gov_info.vote_changed_event,
            VoteChangedEvent {
                proposal_id,
                proposer: proposer_address,
                voter: Signer::address_of(signer),
                agree,
                vote: stake_value,
            },
        );
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
            assert(state == ACTIVE, ERR_PROPOSAL_STATE_INVALID);
        };
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        assert(proposal.id == proposal_id, ERR_PROPOSAL_ID_MISMATCH);
        let my_vote = borrow_global_mut<Vote<TokenT>>(Signer::address_of(signer));
        assert(my_vote.proposer == proposer_address, ERR_PROPOSER_MISMATCH);
        assert(my_vote.id == proposal_id, ERR_PROPOSAL_ID_MISMATCH);
        let reverted_stake = Token::withdraw(&mut my_vote.stake, voting_power);
        if (my_vote.agree) {
            proposal.for_votes = proposal.for_votes - voting_power;
        } else {
            proposal.against_votes = proposal.against_votes - voting_power;
        };
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

    public fun proposal_exists<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ): bool acquires Proposal {
        if (exists<Proposal<TokenT, ActionT>>(proposer_address)) {
            let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
            if (proposal.id == proposal_id) {
                return true
            };
        };
        false
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
            assert(state > ACTIVE, ERR_PROPOSAL_STATE_INVALID);
        };
        let Vote { proposer, id, stake, agree: _ } = move_from<Vote<TokenT>>(
            Signer::address_of(signer),
        );
        // these checks are still required.
        assert(proposer == proposer_address, ERR_PROPOSER_MISMATCH);
        assert(id == proposal_id, ERR_PROPOSAL_ID_MISMATCH);
        stake
    }

    /// queue agreed proposal to execute.
    public fun queue_proposal_action<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires Proposal {
        // Only agreed proposal can be submitted.
        assert(proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == AGREED, 601);
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        proposal.eta = Timestamp::now_seconds() + proposal.action_delay;
    }

    /// extract proposal action to execute.
    public fun extract_proposal_action<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ): ActionT acquires Proposal {
        // Only executable proposal's action can be extracted.
        assert(
            proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == EXECUTABLE,
            ERR_PROPOSAL_STATE_INVALID,
        );
        let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
        let action: ActionT = Option::extract(&mut proposal.action);
        action
    }

    /// remove terminated proposal from proposer
    public fun destroy_terminated_proposal<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires Proposal {
        let proposal_state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
        assert(
            proposal_state == DEFEATED || proposal_state == EXTRACTED,
            ERR_PROPOSAL_STATE_INVALID,
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

    public fun proposal_state<TokenT: copyable, ActionT>(
        proposer_address: address,
        proposal_id: u64,
    ): u8 acquires Proposal {
        let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
        assert(proposal.id == proposal_id, ERR_PROPOSAL_ID_MISMATCH);
        let current_time = Timestamp::now_seconds();
        if (current_time < proposal.start_time) {
            // Pending
            PENDING
        } else if (current_time <= proposal.end_time) {
            // Active
            ACTIVE
        } else if (proposal.for_votes <= proposal.against_votes ||
            proposal.for_votes < quorum_votes<TokenT>()) {
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
        assert(proposal.id == proposal_id, ERR_PROPOSAL_ID_MISMATCH);
        (proposal.start_time, proposal.end_time, proposal.for_votes, proposal.against_votes)
    }

    /// Get voter's vote info on proposal with `proposal_id` of `proposer_address`.
    public fun vote_of<TokenT: copyable>(
        voter: address,
        proposer_address: address,
        proposal_id: u64,
    ): (bool, u128) acquires Vote {
        let vote = borrow_global<Vote<TokenT>>(voter);
        assert(vote.proposer == proposer_address, ERR_PROPOSER_MISMATCH);
        assert(vote.id == proposal_id, ERR_PROPOSAL_ID_MISMATCH);
        (vote.agree, Token::value(&vote.stake))
    }

    /// Quorum votes to make proposal pass.
    public fun quorum_votes<TokenT: copyable>(): u128 {
        let supply = Token::market_cap<TokenT>();
        supply / 100 * (voting_quorum_rate<TokenT>() as u128)
    }

    fun generate_next_proposal_id<TokenT>(): u64 acquires DaoGlobalInfo {
        let gov_info = borrow_global_mut<DaoGlobalInfo<TokenT>>(Token::token_address<TokenT>());
        let proposal_id = gov_info.next_proposal_id;
        gov_info.next_proposal_id = proposal_id + 1;
        proposal_id
    }

    //// Helper functions

    //// Query functions
    public fun voting_delay<TokenT: copyable>(): u64 {
        get_config<TokenT>().voting_delay
    }

    public fun voting_period<TokenT: copyable>(): u64 {
        get_config<TokenT>().voting_period
    }

    public fun voting_quorum_rate<TokenT: copyable>(): u8 {
        get_config<TokenT>().voting_quorum_rate
    }

    public fun min_action_delay<TokenT: copyable>(): u64 {
        get_config<TokenT>().min_action_delay
    }

    fun get_config<TokenT: copyable>(): DaoConfig<TokenT> {
        let token_issuer = Token::token_address<TokenT>();
        Config::get_by_address<DaoConfig<TokenT>>(token_issuer)
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
            assert(voting_quorum_rate <= 100, ERR_QUROM_RATE_INVALID);
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
        assert(value > 0, ERR_CONFIG_PARAM_INVALID);
        let config = get_config<TokenT>();
        config.voting_delay = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    public fun set_voting_period<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert(value > 0, ERR_CONFIG_PARAM_INVALID);
        let config = get_config<TokenT>();
        config.voting_period = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    public fun set_voting_quorum_rate<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u8,
    ) {
        assert(value <= 100 && value > 0, ERR_QUROM_RATE_INVALID);
        let config = get_config<TokenT>();
        config.voting_quorum_rate = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }

    public fun set_min_action_delay<TokenT: copyable>(
        cap: &mut Config::ModifyConfigCapability<DaoConfig<TokenT>>,
        value: u64,
    ) {
        assert(value > 0, ERR_CONFIG_PARAM_INVALID);
        let config = get_config<TokenT>();
        config.min_action_delay = value;
        Config::set_with_capability<DaoConfig<TokenT>>(cap, config);
    }
}
}