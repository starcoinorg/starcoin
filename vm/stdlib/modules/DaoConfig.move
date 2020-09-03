address 0x1 {
  module DaoConfig {
    // use 0x1::Config;
    use 0x1::Token;
    use 0x1::Signer;

    resource struct DaoConfig<TokenT> {
      voting_delay: u64,
      voting_period: u64,
      voting_quorum_rate: u8,
      min_action_delay: u64,
    }

    public fun plugin<TokenT>(
      signer: &signer,
      voting_delay: u64,
      voting_period: u64,
      voting_quorum_rate: u8,
      min_action_delay: u64,
    ) {
      let token_issuer = Token::token_address<TokenT>();
      assert(Signer::address_of(signer) == token_issuer, 401);
      let config = DaoConfig<TokenT> {
        voting_delay,voting_period, voting_quorum_rate, min_action_delay
      };
      move_to(signer, config);
    }



    //// Query functions

    public fun voting_delay<TokenT>(): u64 acquires DaoConfig {
      let token_issuer = Token::token_address<TokenT>();
      borrow_global<DaoConfig<TokenT>>(token_issuer).voting_delay
    }
    public fun voting_period<TokenT>(): u64 acquires DaoConfig {
      let token_issuer = Token::token_address<TokenT>();
      borrow_global<DaoConfig<TokenT>>(token_issuer).voting_period
    }
    public fun voting_quorum_rate<TokenT>(): u8 acquires DaoConfig {
      let token_issuer = Token::token_address<TokenT>();
      borrow_global<DaoConfig<TokenT>>(token_issuer).voting_quorum_rate
    }
    public fun min_action_delay<TokenT>(): u64 acquires DaoConfig {
      let token_issuer = Token::token_address<TokenT>();
      borrow_global<DaoConfig<TokenT>>(token_issuer).min_action_delay
    }
  }
}