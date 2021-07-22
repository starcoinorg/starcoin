module 0x1::SignerManagingProposal {
    use 0x1::Account;
    use 0x1::Token;
    // use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::Dao;
    const ERR_NOT_AUTHORIZED: u64 = 401;

    struct WrappedSignerCapability has key {
        cap: Account::SignerCapability,
    }

    struct BorrowSignerProposal<Borrower> has drop, store, copy { allow: bool }

    /// Plugin in this module to manage token address signer capability using DAO.
    public fun plugin<TokenT: store>(signer_cap: Account::SignerCapability) {
        let token_issuer = Token::token_address<TokenT>();
        assert(Account::signer_address(&signer_cap) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));

        move_to(&Account::borrow_signer_with_capability(&signer_cap), WrappedSignerCapability{cap: signer_cap});
    }

    /// Propose `Borrower` can be borrow signer using `Account::borrow_signer(borrower, signer_address)`
    public fun propose_borrow_signer<TokenT: drop + store + copy, Borrower: drop + store + copy>(signer: &signer, allow: bool, exec_delay: u64) {
        Dao::propose<TokenT, BorrowSignerProposal<Borrower>>(signer, BorrowSignerProposal<Borrower> { allow }, exec_delay);
    }

    /// Execute the borrow_signer proposal.
    public fun execute_borrow_signer_proposal<TokenT: drop + store + copy, Borrower: drop + store + copy>(proposer_address: address, proposal_id: u64)
    acquires WrappedSignerCapability {
        let BorrowSignerProposal<Borrower> { allow } = Dao::extract_proposal_action<TokenT, BorrowSignerProposal<Borrower>>(
            proposer_address,
            proposal_id,
        );
        let cap = borrow_global<WrappedSignerCapability>(Token::token_address<TokenT>());
        if (allow) {
            Account::allow_borrow_signer<Borrower>(&cap.cap);
        } else {
            Account::disallow_borrow_signer<Borrower>(&cap.cap);
        };
    }

}