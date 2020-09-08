//! account: admin
//! account: exchanger

//! new-transaction
//! sender: admin
module Token1 {
    struct Token1 {}
}
// check: EXECUTED


//! new-transaction
//! sender: admin

// register a token pair STC/Token1
script {
    use {{admin}}::Token1;
    use 0x1::Token;
    use 0x1::Account;
    fun main(signer: &signer) {
        Token::register_token<Token1::Token1>(
            signer,
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
        );
        let token = Token::mint<Token1::Token1>(signer, 10000 * 10000 * 2);
        Account::deposit(signer, token);
        assert(Account::balance<Token1::Token1>({{admin}}) == 10000 * 10000 * 2, 42);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
    use 0x1::TokenSwap;
    use {{admin}}::Token1;
    use 0x1::STC;
    fun main(signer: &signer) {
        if (TokenSwap::compare_token<STC::STC, Token1::Token1>() == 1) {
            TokenSwap::register_swap_pair<STC::STC, Token1::Token1>(signer);
        } else {
            TokenSwap::register_swap_pair<Token1::Token1, STC::STC>(signer);
        }
    }
}
// check: EXECUTED

//! new-transaction
//! sender: admin
script {
    use 0x1::TokenSwapGateway;
    use 0x1::STC;
    use {{admin}}::Token1;
    fun add_liquidity(signer: &signer) {
        TokenSwapGateway::add_liquidity<STC::STC, Token1::Token1>(signer, 10000, 10000 * 10000, 0, 0);
        let total_liquidity = TokenSwapGateway::total_liquidity<STC::STC, Token1::Token1>();
        assert(total_liquidity == 1000000 - 1000, (total_liquidity as u64));
        TokenSwapGateway::add_liquidity<STC::STC, Token1::Token1>(signer, 10000, 10000 * 10000, 0, 0);
        let total_liquidity = TokenSwapGateway::total_liquidity<STC::STC, Token1::Token1>();
        assert(total_liquidity == (1000000 - 1000)*2, (total_liquidity as u64));
    }
}
// check: EXECUTED

//! new-transaction
//! sender: admin
script {
    use 0x1::TokenSwapGateway;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Signer;
    fun remove_liquidity(signer: &signer) {
        TokenSwapGateway::remove_liquidity<STC::STC, Token1::Token1>(signer, 10000, 0, 0);
        let token_balance = Account::balance<Token1::Token1>(Signer::address_of(signer));
        let expected = (10000 * 10000) * 2 * 10000 / ((1000000 - 1000)*2);
        assert(token_balance == expected, (token_balance as u64));

        let (stc_reserve, token_reserve) = TokenSwapGateway::get_reserves<STC::STC, Token1::Token1>();
        assert(stc_reserve == 10000 * 2 - 10000 * 2 * 10000 / ((1000000 - 1000)*2), (stc_reserve as u64));
        assert(token_reserve == 10000 * 10000 * 2 - expected, (token_reserve as u64));
    }
}
// check: EXECUTED

//! new-transaction
//! sender: exchanger
script {
    use 0x1::TokenSwapGateway;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Signer;

    fun swap_exact_token_for_token(signer: &signer) {
        let (stc_reserve, token_reserve) = TokenSwapGateway::get_reserves<STC::STC, Token1::Token1>();
        TokenSwapGateway::swap_exact_token_for_token<STC::STC, Token1::Token1>(signer, 1000, 0);
        let token_balance = Account::balance<Token1::Token1>(Signer::address_of(signer));
        let expected_token_balance = TokenSwapGateway::get_amount_out(1000, stc_reserve, token_reserve);
        assert(token_balance == expected_token_balance, (token_balance as u64));
    }
}
// check: EXECUTED

//! new-transaction
//! sender: exchanger
script {
    use 0x1::TokenSwapGateway;
    use 0x1::STC;
    use {{admin}}::Token1;
    use 0x1::Account;
    use 0x1::Signer;
    fun swap_token_for_exact_token(signer: &signer) {
        let stc_balance_before = Account::balance<STC::STC>(Signer::address_of(signer));
        let (stc_reserve, token_reserve) = TokenSwapGateway::get_reserves<STC::STC, Token1::Token1>();
        TokenSwapGateway::swap_token_for_exact_token<STC::STC, Token1::Token1>(signer, 30, 100000);
        let stc_balance_after = Account::balance<STC::STC>(Signer::address_of(signer));

        let expected_balance_change = TokenSwapGateway::get_amount_in(100000, stc_reserve, token_reserve);
        assert(stc_balance_before - stc_balance_after == expected_balance_change, (expected_balance_change as u64));
    }
}
// check: EXECUTED

