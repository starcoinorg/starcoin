address 0x1 {
  module Dao {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Block;
    use 0x1::Option;

    /// make them into configs
    const VOTEING_DELAY: u64 = 100;
    const VOTEING_PERIOD: u64 = 200;
    /// quorum rate: 4% of toal token supply.
    const VOTEING_QUORUM_RATE: u8 = 4;

    const MIN_ACTION_DELAY: u64 = 200;

    /// Proposal state
    const PENDING: u8 = 1;
    const ACTIVE: u8 = 2;
    const DEFEATED: u8 = 3;
    const AGREED: u8 = 4;
    const QUEUED: u8 = 5;
    const EXECUTABLE: u8 = 6;
    const EXTRACTED: u8 = 7;

    resource struct GovGlobalInfo<Token> {
      next_proposal_id: u64,
    }

    /// TODO: support that one can propose mutli proposals.
    resource struct Proposal<Token, Action> {
      id: u64,
      proposer: address,
      start_block: u64,
      end_block: u64,
      for_votes: u128,
      against_votes: u128,
      // executable after this block.
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


    /// plug_in function, can only be called by token issuer.
    /// Any token who wants to has gov functionality
    /// can optin this moudle by call this `register function`.
    public fun plugin<TokenT>(signer: &signer) {
      // TODO: we can add a token manage cap in Token module.
      // and only token manager can register this.
      let token_issuer = Token::token_address<TokenT>();
      assert(Signer::address_of(signer) == token_issuer, 401);
      // let proposal_id = ProposalId {next: 0};
      let gov_info = GovGlobalInfo<TokenT> {
        next_proposal_id: 0,
      };
      move_to(signer, gov_info);
    }

    /// propose a proposal.
    /// `action`: the actual action to execute.
    /// `action_delay`: the delay to execute after the proposal is agreed
    public fun propose<TokenT, ActionT>(signer: &signer, action: ActionT, action_delay: u64)
    acquires GovGlobalInfo {
      assert(action_delay >= MIN_ACTION_DELAY, 401);
      let proposal_id = generate_next_proposal_id<TokenT>();
      // TODO: make the delay configurable
      let start_block = Block::get_current_block_number() + VOTEING_DELAY;
      let proposal = Proposal<TokenT, ActionT> {
        id: proposal_id,
        proposer: Signer::address_of(signer),
        start_block: start_block,
        end_block: start_block + VOTEING_PERIOD,
        for_votes: 0,
        against_votes: 0,
        eta: 0,
        action_delay,
        action: Option::some(action),
      };

      move_to(signer, proposal);
    }

    /// votes for a proposal.
    /// User can only vote once, then the stake is locked,
    /// which can only be unstaked by user after the proposal is expired, or cancelled, or executed.
    /// So think twice before casting vote.
    public fun cast_vote<TokenT, ActionT>(signer: &signer, proposer_address: address, proposal_id: u64, stake: Token::Token<TokenT>, agree: bool)
    acquires Proposal {
      {
        let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
        assert(state <= ACTIVE, 700);
      };

      let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
      assert(proposal.id == proposal_id, 500);
      let stake_value = Token::value(&stake);
      let my_vote = Vote<TokenT> {
        proposer: proposer_address,
        id: proposal_id,
        stake: stake,
        agree,
      };
      if (agree) {
        proposal.for_votes = proposal.for_votes + stake_value;
      } else {
        proposal.against_votes = proposal.against_votes + stake_value;
      };

      move_to(signer, my_vote);
    }

    /// Retrieve back my staked token voted for a proposal.
    public fun unstake_votes<TokenT, ActionT>(signer: &signer, proposer_address: address, proposal_id: u64): Token::Token<TokenT>
    acquires Proposal, Vote {
      {
        let state = proposal_state<TokenT, ActionT>(proposer_address, proposal_id);
        // Only after vote period end, user can unstake his votes.
        assert(state > ACTIVE, 800);
      };
      let Vote {
        proposer,
        id,
        stake,
        agree: _,
      } = move_from<Vote<TokenT>>(Signer::address_of(signer));
      assert(proposer == proposer_address, 100);
      assert(id == proposal_id, 101);
      stake
    }

    /// queue agreed proposal to execute.
    public fun queue_proposal_action<TokenT, ActionT>(proposer_address: address, proposal_id: u64)
    acquires Proposal {
      // Only agreed proposal can be submitted.
      assert(proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == AGREED, 601);
      let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
      proposal.eta = Block::get_current_block_number() + proposal.action_delay;
    }

    /// extract proposal action to execute.
    public fun extract_proposal_action<TokenT, ActionT>(proposer_address: address, proposal_id: u64): ActionT
    acquires Proposal {
      // Only executable proposal's action can be extracted.
      assert(proposal_state<TokenT, ActionT>(proposer_address, proposal_id) == EXECUTABLE, 601);
      let proposal = borrow_global_mut<Proposal<TokenT, ActionT>>(proposer_address);
      let action: ActionT = Option::extract(&mut proposal.action);
      action
    }

    fun proposal_state<TokenT, ActionT>( proposer_address: address, proposal_id: u64): u8
    acquires Proposal {
      let proposal = borrow_global<Proposal<TokenT, ActionT>>(proposer_address);
      assert(proposal.id == proposal_id, 500);
      let current_block_number = Block::get_current_block_number();
      if (current_block_number <= proposal.start_block) {
        // Pending
        PENDING
      } else if (current_block_number <= proposal.end_block) {
        // Active
        ACTIVE
      } else if (proposal.for_votes <= proposal.against_votes || proposal.for_votes < quorum_votes<TokenT>()) {
        // Defeated
        DEFEATED
      } else if (proposal.eta == 0) {
        // Agreed.
        AGREED
      } else if (proposal.eta < current_block_number) {
        // Queued, waiting to execute
        QUEUED
      } else if (Option::is_some(&proposal.action)) {
        EXECUTABLE
      } else {
        EXTRACTED
      }
    }

    /// Quorum votes to make proposal pass.
    public fun quorum_votes<TokenT>(): u128 {
      let supply = Token::market_cap<TokenT>();
      supply / 100 * (VOTEING_QUORUM_RATE as u128)
    }

    fun generate_next_proposal_id<TokenT>(): u64 acquires GovGlobalInfo {
      let gov_info = borrow_global_mut<GovGlobalInfo<TokenT>>(Token::token_address<TokenT>());
      let proposal_id = gov_info.next_proposal_id;
      gov_info.next_proposal_id = proposal_id + 1;
      proposal_id
    }
  }
}