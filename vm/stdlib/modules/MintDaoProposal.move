address 0x1 {
module MintDaoProposal {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Dao;
    use 0x1::Account;

    resource struct WrappedMintCapability<TokenType> {
        cap: Token::MintCapability<TokenType>,
    }

    struct MintToken {
        receiver: address,
        amount: u128,
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;

    public fun plugin<TokenT>(signer: &signer) {
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, ERR_NOT_AUTHORIZED);
        let mint_cap = Token::remove_mint_capability<TokenT>(signer);
        move_to(signer, WrappedMintCapability { cap: mint_cap });
    }

    public fun propose_mint_to<TokenT: copyable>(signer: &signer, receiver: address, amount: u128, exec_delay: u64) {
        Dao::propose<TokenT, MintToken>(
            signer,
            MintToken { receiver, amount },
            exec_delay,
        );
    }

    public fun execute_mint_proposal<TokenT: copyable>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) acquires WrappedMintCapability {
        let MintToken { receiver, amount } = Dao::extract_proposal_action<TokenT, MintToken>(
            proposer_address,
            proposal_id,
        );
        let cap = borrow_global<WrappedMintCapability<TokenT>>(Token::token_address<TokenT>());
        let tokens = Token::mint_with_capability<TokenT>(&cap.cap, amount);
        Account::deposit_to(signer, receiver, tokens);
    }
}
}