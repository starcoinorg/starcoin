// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

address 0x1 {
module YieldFarmingTreasury {
    use 0x1::Token::{Token, Self};
    use 0x1::Signer;
    use 0x1::Errors;

    struct Treasury<PoolType, TokenT> has store, key {
        balance: Token<TokenT>,
    }

    const ERR_INVALID_PERIOD: u64 = 101;
    const ERR_ZERO_AMOUNT: u64 = 102;
    const ERR_TOO_BIG_AMOUNT: u64 = 103;
    const ERR_NOT_AUTHORIZED: u64 = 104;
    const ERR_TREASURY_NOT_EXIST: u64 = 105;

    struct WithdrawCapability<PoolType, GovTokenT> has key, store {}

    /// Init a Treasury for TokenT,can only be called by token issuer.
    public fun initialize<PoolType: store,
                          TokenT: store>(
        signer: &signer,
        init_token: Token::Token<TokenT>): WithdrawCapability<PoolType, TokenT> {
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let treasure = Treasury<PoolType, TokenT> { balance: init_token, };
        move_to(signer, treasure);
        WithdrawCapability<PoolType, TokenT> {}
    }

    /// Withdraw from TokenT's treasury with WithdrawCapability reference
    public fun withdraw_with_capability<
        PoolType: store, TokenT: store>(_cap: &WithdrawCapability<PoolType, TokenT>,
                                        amount: u128): Token::Token<TokenT> acquires Treasury {
        assert(amount > 0, Errors::invalid_argument(ERR_ZERO_AMOUNT));
        assert(exists_at<PoolType, TokenT>(), Errors::not_published(ERR_TREASURY_NOT_EXIST));
        let token_address = Token::token_address<TokenT>();
        let treasury = borrow_global_mut<Treasury<PoolType, TokenT>>(token_address);
        assert(amount <= Token::value(&treasury.balance), Errors::invalid_argument(ERR_TOO_BIG_AMOUNT));
        Token::withdraw(&mut treasury.balance, amount)
    }

    /// Get the balance of TokenT's Treasury
    /// if the Treasury do not exists, return 0.
    public fun balance<PoolType: store, TokenT: store>(): u128 acquires Treasury {
        let token_issuer = Token::token_address<TokenT>();
        if (!exists<Treasury<PoolType, TokenT>>(token_issuer)) {
            return 0
        };
        let treasury = borrow_global<Treasury<PoolType, TokenT>>(token_issuer);
        Token::value(&treasury.balance)
    }

    public fun deposit<PoolType: store, TokenT: store>(token: Token<TokenT>) acquires Treasury {
        assert(exists_at<PoolType, TokenT>(), Errors::not_published(ERR_TREASURY_NOT_EXIST));
        let token_address = Token::token_address<TokenT>();
        let treasury = borrow_global_mut<Treasury<PoolType, TokenT>>(token_address);
        //let amount = Token::value(&token);
        Token::deposit(&mut treasury.balance, token);
    }

    /// Check the Treasury of TokenT is exists.
    public fun exists_at<PoolType: store, TokenT: store>(): bool {
        let token_issuer = Token::token_address<TokenT>();
        exists<Treasury<PoolType, TokenT>>(token_issuer)
    }
}
}