address 0x1 {
  module ModifyDaoConfigProposal {
    // use 0x1::Config;
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Config;
    use 0x1::Option;
    use 0x1::Dao;

    resource struct DaoConfigModifyCapability<TokenT: copyable> {
      cap: Config::ModifyConfigCapability<Dao::DaoConfig<TokenT>>,
    }

    /// a proposal action to udpate dao config.
    /// if any field is some, that means the proposal want to update.
    struct DaoConfigUpdate {
      voting_delay: Option::Option<u64>,
      voting_period: Option::Option<u64>,
      voting_quorum_rate: Option::Option<u8>,
      min_action_delay: Option::Option<u64>,
    }

    public fun plugin<TokenT: copyable>(
      signer: &signer,
    ) {
      let token_issuer = Token::token_address<TokenT>();
      assert(Signer::address_of(signer) == token_issuer, 401);
      let dao_config_moidify_cap = Config::extract_modify_config_capability<Dao::DaoConfig<TokenT>>(signer);
      // TODO: assert cap.account_address == token_issuer
      let cap = DaoConfigModifyCapability {
        cap: dao_config_moidify_cap,
      };
      move_to(signer, cap);
    }


    public fun propose<TokenT: copyable>(
      signer: &signer,
      voting_delay: Option::Option<u64>,
      voting_period: Option::Option<u64>,
      voting_quorum_rate: Option::Option<u8>,
      min_action_delay: Option::Option<u64>,
    ) {
      let action = DaoConfigUpdate {
        voting_delay, voting_period, voting_quorum_rate, min_action_delay
      };
      Dao::propose<TokenT, DaoConfigUpdate>(signer, action, Dao::min_action_delay<TokenT>());
    }

    public fun execute<TokenT: copyable>(
      proposer_address: address, proposal_id: u64
     ) acquires DaoConfigModifyCapability {
      let DaoConfigUpdate {
        voting_delay,
        voting_period,
        voting_quorum_rate,
        min_action_delay,
      } = Dao::extract_proposal_action<TokenT, DaoConfigUpdate>(proposer_address, proposal_id);
      let cap = borrow_global_mut<DaoConfigModifyCapability<TokenT>>(Token::token_address<TokenT>());
      Dao::modify_dao_config(&mut cap.cap, voting_delay, voting_period, voting_quorum_rate, min_action_delay);
    }
  }
}