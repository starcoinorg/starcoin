address 0x1 {
  module Gov {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::PackageTxnManager;
    use 0x1::Vector;
    use 0x1::Block;
    use 0x1::Option;
    use 0x1::Account;

    /// make them into configs
    const VOTEING_DELAY: u64 = 100;
    const VOTEING_PERIOD: u64 = 200;
    /// quorum rate: 4% of toal token supply.
    const VOTEING_QUORUM_RATE: u8 = 4;

    const EXECUTE_DELAY: u64 = 200;

    /// Proposal state
    const PENDING: u8 = 1;
    const ACTIVE: u8 = 2;
    const DEFEATED: u8 = 3;
    const AGREED: u8 = 4;
    const QUEUED: u8 = 5;


    resource struct GovGlobalInfo<Token> {
      next_proposal_id: u64,
      caps: vector<WrappedUpgradePlanCapability>,
    }

    /// TODO: support that one can propose mutli proposals.
    resource struct Proposal<Token> {
      info: ProposalInfo,
      // TODO: How to handle other proposal actions?
      upgrade_module: Option::Option<UpgradeModule>,
    }

    struct UpgradeModule {
      module_address: address,
      module_hash: vector<u8>,
    }

    struct ProposalInfo {
      id: u64,
      proposer: address,
      start_block: u64,
      end_block: u64,
      for_votes: u128,
      against_votes: u128,
      // executable after this block.
      eta: u64,
    }

    resource struct WrappedUpgradePlanCapability {
      cap: PackageTxnManager::UpgradePlanCapability,
    }

    // TODO: allow user do multi votes.
    resource struct Vote<TokenT> {
      proposer: address,
      id: u64,
      stake: Token::Token<TokenT>,
      agree: bool,
    }


    /// init function, can only be called by token issuer.
    /// Any token who wants to has gov functionality
    /// can optin this moudle by call this `register function`.
    public fun register<TokenT>(signer: &signer) {
      // TODO: we can add a token manage cap in Token module.
      // and only token manager can register this.
      let token_issuer = Token::token_address<TokenT>();
      assert(Signer::address_of(signer) == token_issuer, 401);
      // let proposal_id = ProposalId {next: 0};
      let gov_info = GovGlobalInfo<TokenT> {
        next_proposal_id: 0,
        caps: Vector::empty(),
      };
      move_to(signer, gov_info);
    }

    /// If this govverment can upgrade module, call this to register capability.
    public fun delegate_module_upgrade_capability<TokenT>(signer: &signer, cap: PackageTxnManager::UpgradePlanCapability)
    acquires GovGlobalInfo {
      let token_issuer = Token::token_address<TokenT>();
      assert(Signer::address_of(signer) == token_issuer, 401);
      let gov_info = borrow_global_mut<GovGlobalInfo<TokenT>>(token_issuer);
      // TODO: should check duplicate cap?
      // for now, only one cap exists for a module address.
      Vector::push_back(&mut gov_info.caps, WrappedUpgradePlanCapability {cap});
    }

    /// check whether this gov has the ability to upgrade module in `moudle_address`.
    public fun able_to_upgrade<TokenT>(module_address: address): bool acquires GovGlobalInfo {
      let pos = find_module_upgrade_cap<TokenT>(module_address);
      Option::is_some(&pos)
    }

    fun find_module_upgrade_cap<TokenT>(module_address: address): Option::Option<u64> acquires GovGlobalInfo {
      let token_issuer = Token::token_address<TokenT>();
      let gov_info = borrow_global<GovGlobalInfo<TokenT>>(token_issuer);
      let cap_len = Vector::length(&gov_info.caps);
      let i = 0;
      while (i < cap_len) {
        let cap = Vector::borrow(&gov_info.caps, i);
        let account_address = PackageTxnManager::account_address(&cap.cap);
        if (account_address == module_address) {
          return Option::some(i)
        };
        i = i + 1;
      };
      Option::none<u64>()
    }

    /// propose a module upgrade, called by proposer.
    public fun propose_module_upgrade<TokenT>(signer: &signer, module_address: address, module_hash: vector<u8>)
    acquires GovGlobalInfo {
      assert(able_to_upgrade<TokenT>(module_address), 400);
      let proposal_id = generate_next_proposal_id<TokenT>();
      // TODO: make the delay configurable
      let start_block = Block::get_current_block_number() + VOTEING_DELAY;
      let proposal = Proposal<TokenT> {
        info: ProposalInfo {
          id: proposal_id,
          proposer: Signer::address_of(signer),
          start_block: start_block,
          end_block: start_block + VOTEING_PERIOD,
          for_votes: 0,
          against_votes: 0,
          eta: 0,
        },
        upgrade_module: Option::some(UpgradeModule {
          module_address,
          module_hash
        })
      };

      move_to(signer, proposal);
    }

    /// votes for a proposal.
    /// User can only vote once, then the stake is locked,
    /// which can only be unstaked by user after the proposal is expired, or cancelled, or executed.
    /// So think twice before casting vote.
    public fun cast_vote<TokenT>(signer: &signer, proposer_address: address, proposal_id: u64, stake: u128, agree: bool)
    acquires Proposal {
      {
        let state = proposal_state<TokenT>(proposer_address, proposal_id);
        assert(state <= ACTIVE, 700);
      };

      let proposal = borrow_global_mut<Proposal<TokenT>>(proposer_address);
      assert(proposal.info.id == proposal_id, 500);
      let stakes = Account::withdraw<TokenT>(signer, stake);
      let my_vote = Vote<TokenT> {
        proposer: proposer_address,
        id: proposal_id,
        stake: stakes,
        agree,
      };
      if (agree) {
        proposal.info.for_votes = proposal.info.for_votes + stake;
      } else {
        proposal.info.against_votes = proposal.info.against_votes + stake;
      };

      move_to(signer, my_vote);
    }

    /// Retrieve back my staked token voted for a proposal.
    public fun unstake_votes<TokenT>(signer: &signer, proposer_address: address, proposal_id: u64)
    acquires Proposal, Vote {
      {
        let state = proposal_state<TokenT>(proposer_address, proposal_id);
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
      Account::deposit(signer, stake);
    }

    public fun submit_proposed_action<TokenT>(_signer: &signer, proposer_address: address, proposal_id: u64)
    acquires Proposal, GovGlobalInfo {
      // Only agreed proposal can be submitted.
      assert(proposal_state<TokenT>(proposer_address, proposal_id) == AGREED, 601);

      let proposal = borrow_global_mut<Proposal<TokenT>>(proposer_address);
      assert(proposal.info.id == proposal_id, 500);
      if (Option::is_some(&proposal.upgrade_module)) {
        let UpgradeModule {
          module_address,
          module_hash
        } = Option::extract(&mut proposal.upgrade_module);
        let eta = Block::get_current_block_number() + EXECUTE_DELAY;
        proposal.info.eta = eta;
        submit_upgrade_plan<TokenT>(module_address, module_hash, eta);
      } else {
        assert(false, 700);
      }
    }

    fun submit_upgrade_plan<TokenT>(module_address: address, module_hash: vector<u8>, eta: u64) acquires GovGlobalInfo {
      let pos = find_module_upgrade_cap<TokenT>(module_address);
      assert(Option::is_some(&pos), 500);
      let pos = Option::extract(&mut pos);
      let gov_info = borrow_global<GovGlobalInfo<TokenT>>(Token::token_address<TokenT>());
      let cap = Vector::borrow(&gov_info.caps, pos);
      PackageTxnManager::submit_upgrade_plan_with_cap(&cap.cap, module_hash, eta);
    }

    fun proposal_state<TokenT>( proposer_address: address, proposal_id: u64): u8
    acquires Proposal {
      let proposal = borrow_global<Proposal<TokenT>>(proposer_address);
      assert(proposal.info.id == proposal_id, 500);
      let current_block_number = Block::get_current_block_number();
      if (current_block_number <= proposal.info.start_block) {
        // Pending
        PENDING
      } else if (current_block_number <= proposal.info.end_block) {
        // Active
        ACTIVE
      } else if (proposal.info.for_votes <= proposal.info.against_votes || proposal.info.for_votes < quorum_votes<TokenT>()) {
        // Defeated
        DEFEATED
      } else if (proposal.info.eta == 0) {
        // Agreed.
        AGREED
      } else {
        // Queued, waiting to execute
        QUEUED
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