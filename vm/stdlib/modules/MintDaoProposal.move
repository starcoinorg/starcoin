address 0x1 {
module MintDaoProposal {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Dao;
    use 0x1::Account;
    use 0x1::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

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
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let mint_cap = Token::remove_mint_capability<TokenT>(signer);
        move_to(signer, WrappedMintCapability { cap: mint_cap });
    }
    spec fun plugin {
        pragma aborts_if_is_partial = false;
        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<Token::MintCapability<TokenT>>(sender);
        aborts_if exists<WrappedMintCapability<TokenT>>(sender);

        ensures !exists<Token::MintCapability<TokenT>>(sender);
        ensures exists<WrappedMintCapability<TokenT>>(sender);
    }


    public fun propose_mint_to<TokenT: copyable>(signer: &signer, receiver: address, amount: u128, exec_delay: u64) {
        Dao::propose<TokenT, MintToken>(
            signer,
            MintToken { receiver, amount },
            exec_delay,
        );
    }
    spec fun propose_mint_to {
        use 0x1::Timestamp;
        use 0x1::CoreAddresses;
        pragma aborts_if_is_partial = false;

        // copy from Dao::propose spec.
        include Dao::AbortIfDaoConfigNotExist<TokenT>;
        include Dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if exec_delay > 0 && exec_delay < Dao::spec_dao_config<TokenT>().min_action_delay;
        include Dao::CheckQuorumVotes<TokenT>;
        let sender = Signer::spec_address_of(signer);
        aborts_if exists<Dao::Proposal<TokenT, MintToken>>(sender);
    }

    public fun execute_mint_proposal<TokenT: copyable>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires WrappedMintCapability {
        let MintToken { receiver, amount } = Dao::extract_proposal_action<TokenT, MintToken>(
            proposer_address,
            proposal_id,
        );
        let cap = borrow_global<WrappedMintCapability<TokenT>>(Token::token_address<TokenT>());
        let tokens = Token::mint_with_capability<TokenT>(&cap.cap, amount);
        Account::deposit(receiver, tokens);
    }

    spec fun execute_mint_proposal {
        use 0x1::Option;
        pragma aborts_if_is_partial = true;
        let expected_states = singleton_vector(6);
        include Dao::CheckProposalStates<TokenT, MintToken>{expected_states};
        let proposal = global<Dao::Proposal<TokenT, MintToken>>(proposer_address);
        aborts_if Option::spec_is_none(proposal.action);
        aborts_if !exists<WrappedMintCapability<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }
}
}