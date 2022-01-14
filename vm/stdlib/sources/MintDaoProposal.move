address StarcoinFramework {
/// MintDaoProposal is a dao proposal for mint extra tokens.
module MintDaoProposal {
    use StarcoinFramework::Token;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Dao;
    use StarcoinFramework::Account;
    use StarcoinFramework::Errors;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    /// A wrapper of Token MintCapability.
    struct WrappedMintCapability<phantom TokenType> has key {
        cap: Token::MintCapability<TokenType>,
    }

    /// MintToken request.
    struct MintToken has copy, drop, store {
        /// the receiver of minted tokens.
        receiver: address,
        /// how many tokens to mint.
        amount: u128,
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;

    /// Plugin method of the module.
    /// Should be called by token issuer.
    public fun plugin<TokenT: store>(signer: &signer) {
        let token_issuer = Token::token_address<TokenT>();
        assert!(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let mint_cap = Token::remove_mint_capability<TokenT>(signer);
        move_to(signer, WrappedMintCapability { cap: mint_cap });
    }
    spec plugin {
        pragma aborts_if_is_partial = false;
        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<Token::MintCapability<TokenT>>(sender);
        aborts_if exists<WrappedMintCapability<TokenT>>(sender);

        ensures !exists<Token::MintCapability<TokenT>>(sender);
        ensures exists<WrappedMintCapability<TokenT>>(sender);
    }


    /// Entrypoint for the proposal.
    public fun propose_mint_to<TokenT: copy + drop + store>(signer: &signer, receiver: address, amount: u128, exec_delay: u64) {
        Dao::propose<TokenT, MintToken>(
            signer,
            MintToken { receiver, amount },
            exec_delay,
        );
    }
    spec propose_mint_to {
        use StarcoinFramework::Timestamp;
        use StarcoinFramework::CoreAddresses;
        pragma aborts_if_is_partial = false;

        // copy from Dao::propose spec.
        include Dao::AbortIfDaoConfigNotExist<TokenT>;
        include Dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if exec_delay > 0 && exec_delay < Dao::spec_dao_config<TokenT>().min_action_delay;
        include Dao::CheckQuorumVotes<TokenT>;
        let sender = Signer::address_of(signer);
        aborts_if exists<Dao::Proposal<TokenT, MintToken>>(sender);
    }

    /// Once the proposal is agreed, anyone can call the method to make the proposal happen.
    public fun execute_mint_proposal<TokenT: copy + drop + store>(
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

    spec execute_mint_proposal {
        use StarcoinFramework::Option;
        pragma aborts_if_is_partial = true;
        let expected_states = vec<u8>(6);
        include Dao::CheckProposalStates<TokenT, MintToken>{expected_states};
        let proposal = global<Dao::Proposal<TokenT, MintToken>>(proposer_address);
        aborts_if Option::is_none(proposal.action);
        aborts_if !exists<WrappedMintCapability<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }
}
}