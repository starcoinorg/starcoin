//! account: admin
//! account: liquidier
//! account: exchanger


// check: EXECUTED

//! new-transaction
//! sender: admin
module TokenSwapHelper {
  public fun quote(amount_x: u128, reserve_x: u128, reserve_y: u128): u128 {
      assert(amount_x > 0, 400);
      assert(reserve_x > 0 && reserve_y > 0, 410);
      let amount_y = amount_x * reserve_y / reserve_x;
      amount_y
  }

  public fun get_amount_out(amount_in: u128, reserve_in: u128, reserve_out: u128): u128 {
      assert(amount_in > 0, 400);
      assert(reserve_in > 0 && reserve_out > 0, 410);
      let amount_in_with_fee = amount_in * 997;
      let numerator = amount_in_with_fee * reserve_out;
      let denominator = reserve_in * 1000 + amount_in_with_fee;
      (numerator / denominator)
  }

  public fun get_amount_in(amount_out: u128, reserve_in: u128, reserve_out: u128): u128 {
      assert(amount_out > 0, 400);
      assert(reserve_in > 0 && reserve_out > 0, 410);
      let numerator = reserve_in * amount_out * 1000;
      let denominator = (reserve_out - amount_out) * 997;
      (numerator / denominator) + 1
  }
}
// check: EXECUTED

//! new-transaction
//! sender: admin
module LiquidityToken {
    struct LiquidityToken<X, Y> {}
}
// check: EXECUTED

//! new-transaction
//! sender: admin
module TokenSwap {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Math;
    use {{admin}}::LiquidityToken::LiquidityToken;

    // Liquidity Token
    // TODO: token should be generic on <X, Y>
    // resource struct T {
    // }
    resource struct LiquidityTokenCapability<X, Y> {
        mint: Token::MintCapability<LiquidityToken<X, Y>>,
        burn: Token::BurnCapability<LiquidityToken<X, Y>>,
    }

    resource struct TokenPair<X, Y> {
        token_x_reserve: Token::Token<X>,
        token_y_reserve: Token::Token<Y>,
        last_block_timestamp: u64,
        last_price_x_cumulative: u128,
        last_price_y_cumulative: u128,
        last_k: u128,
    }



    /// TODO: check X,Y is token, and X,Y is sorted.


    // for now, only admin can register token pair
    public fun register_swap_pair<X, Y>(signer: &signer) {
        assert_admin(signer);
        let token_pair = make_token_pair<X, Y>();
        move_to(signer, token_pair);
        register_liquidity_token<X, Y>(signer);
    }

    fun register_liquidity_token<X, Y>(signer: &signer) {
        assert_admin(signer);

        Token::register_token<LiquidityToken<X, Y>>(signer, 3);

        let mint_capability = Token::remove_mint_capability<LiquidityToken<X, Y>>(signer);
        let burn_capability = Token::remove_burn_capability<LiquidityToken<X, Y>>(signer);
        move_to(signer, LiquidityTokenCapability {
            mint: mint_capability,
            burn: burn_capability,
        });
    }

    fun make_token_pair<X, Y>(): TokenPair<X, Y> {
        // TODO: assert X, Y is token
        TokenPair<X, Y> {
            token_x_reserve: Token::zero<X>(),
            token_y_reserve: Token::zero<Y>(),
            last_block_timestamp: 0,
            last_price_x_cumulative: 0,
            last_price_y_cumulative: 0,
            last_k: 0,
        }
    }

    /// Liquidity Provider's methods

    public fun mint<X, Y>(x: Token::Token<X>, y: Token::Token<Y>): Token::Token<LiquidityToken<X, Y>>
    acquires TokenPair, LiquidityTokenCapability {
        let total_supply: u128 = Token::market_cap<LiquidityToken<X, Y>>();
        let x_value = Token::value<X>(&x);
        let y_value = Token::value<Y>(&y) ;

        let liquidity = if (total_supply == 0) {
            // 1000 is the MINIMUM_LIQUIDITY
            (Math::sqrt((x_value as u128) * (y_value as u128)) as u128) - 1000
        } else {
            let token_pair = borrow_global<TokenPair<X, Y>>(admin_address());
            let x_reserve = Token::value(&token_pair.token_x_reserve);
            let y_reserve = Token::value(&token_pair.token_y_reserve);

            let x_liquidity = x_value * total_supply / x_reserve;
            let y_liquidity = y_value * total_supply / y_reserve;

            // use smaller one.
            if (x_liquidity < y_liquidity) {
                x_liquidity
            } else {
                y_liquidity
            }
        };
        assert(liquidity > 0, 100);

        let token_pair = borrow_global_mut<TokenPair<X, Y>>(admin_address());
        Token::deposit(&mut token_pair.token_x_reserve, x);
        Token::deposit(&mut token_pair.token_y_reserve, y);

        let liquidity_cap = borrow_global<LiquidityTokenCapability<X, Y>>(admin_address());
        let mint_token = Token::mint_with_capability(&liquidity_cap.mint, liquidity);
        mint_token
    }

    public fun burn<X, Y>(to_burn: Token::Token<LiquidityToken<X, Y>>): (Token::Token<X>, Token::Token<Y>)
    acquires TokenPair, LiquidityTokenCapability {
        let to_burn_value = (Token::value(&to_burn) as u128);

        let token_pair = borrow_global_mut<TokenPair<X, Y>>(admin_address());
        let x_reserve = (Token::value(&token_pair.token_x_reserve) as u128);
        let y_reserve = (Token::value(&token_pair.token_y_reserve) as u128);
        let total_supply = Token::market_cap<LiquidityToken<X, Y>>();

        let x = to_burn_value * x_reserve / total_supply;
        let y = to_burn_value * y_reserve / total_supply;
        assert(x > 0 && y > 0, 101);

        burn_liquidity(to_burn);

        let x_token = Token::withdraw(&mut token_pair.token_x_reserve, x);
        let y_token = Token::withdraw(&mut token_pair.token_y_reserve, y);

        (x_token, y_token)
    }

    fun burn_liquidity<X, Y>(to_burn: Token::Token<LiquidityToken<X, Y>>)
    acquires LiquidityTokenCapability {
        let liquidity_cap = borrow_global<LiquidityTokenCapability<X, Y>>(admin_address());
        Token::burn_with_capability<LiquidityToken<X, Y>>(&liquidity_cap.burn, to_burn);
    }

    /// User methods

    public fun get_reserves<X, Y>(): (u128, u128) acquires TokenPair {
        let token_pair = borrow_global<TokenPair<X, Y>>(admin_address());
        let x_reserve = Token::value(&token_pair.token_x_reserve);
        let y_reserve = Token::value(&token_pair.token_y_reserve);
        (x_reserve, y_reserve)
    }

    public fun swap<X, Y>(x_in: Token::Token<X>, y_out: u128, y_in: Token::Token<Y>, x_out: u128): (Token::Token<X>, Token::Token<Y>)
    acquires TokenPair {
        let x_in_value = Token::value(&x_in);
        let y_in_value = Token::value(&y_in);
        assert(x_in_value > 0 || y_in_value > 0, 400);

        let (x_reserve, y_reserve) = get_reserves<X, Y>();

        let token_pair = borrow_global_mut<TokenPair<X, Y>>(admin_address());
        Token::deposit(&mut token_pair.token_x_reserve, x_in);
        Token::deposit(&mut token_pair.token_y_reserve, y_in);
        let x_swapped = Token::withdraw(&mut token_pair.token_x_reserve, x_out);
        let y_swapped = Token::withdraw(&mut token_pair.token_y_reserve, y_out);

        {
            let x_reserve_new = Token::value(&token_pair.token_x_reserve);
            let y_reserve_new = Token::value(&token_pair.token_y_reserve);
            let x_adjusted = x_reserve_new * 1000 - x_in_value * 3;
            let y_adjusted = y_reserve_new * 1000 - y_in_value * 3;
            assert(x_adjusted * y_adjusted >= x_reserve * y_reserve * 1000000, 500);
        };

        (x_swapped, y_swapped)
    }

    fun assert_admin(signer: &signer) {
        assert(Signer::address_of(signer) == admin_address(), 401);
    }
    fun admin_address(): address {
        {{admin}}
    }
}
// check: EXECUTED

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
use {{admin}}::TokenSwap;
use {{admin}}::Token1;
use 0x1::Token;
use 0x1::STC;
fun main(signer: &signer) {
    Token::register_token<Token1::Token1>(
        signer,
        3,
    );
    TokenSwap::register_swap_pair<STC::STC, Token1::Token1>(signer);
}
}
// check: EXECUTED

//! new-transaction
//! sender: liquidier
script{
use {{admin}}::Token1;
use 0x1::Account;
fun main(signer: &signer) {
    Account::accept_token<Token1::Token1>(signer);
}
}
// check: EXECUTED

//! new-transaction
//! sender: admin
// mint some token1 to liquidier
script{
use {{admin}}::Token1;

use 0x1::Account;
use 0x1::Token;
fun main(signer: &signer) {
    let token = Token::mint<Token1::Token1>(signer, 100000000);
    Account::deposit({{liquidier}}, token);
    assert(Account::balance<Token1::Token1>({{liquidier}}) == 100000000, 42);
}
}

//! new-transaction
//! sender: liquidier
script{
    use 0x1::STC;
    use {{admin}}::Token1;
    use {{admin}}::TokenSwap;
    use {{admin}}::LiquidityToken::LiquidityToken;
    use 0x1::Account;


    fun main(signer: &signer) {
        Account::accept_token<LiquidityToken<STC::STC, Token1::Token1>>(signer);
        // STC/Token1 = 1:10
        let stc_amount = 1000000;
        let token1_amount = 10000000;
        let stc = Account::withdraw<STC::STC>(signer, stc_amount);
        let token1 = Account::withdraw<Token1::Token1>(signer, token1_amount);
        let liquidity_token = TokenSwap::mint<STC::STC, Token1::Token1>(stc, token1);
        Account::deposit_to_self(signer, liquidity_token);

        let (x, y) = TokenSwap::get_reserves<STC::STC, Token1::Token1>();
        assert(x == stc_amount, 111);
        assert(y == token1_amount, 112);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: exchanger
script {
    use 0x1::STC;
    use {{admin}}::Token1;
    use {{admin}}::TokenSwap;
    use {{admin}}::TokenSwapHelper;
    use 0x1::Account;
    use 0x1::Token;
    fun main(signer: &signer) {
        Account::accept_token<Token1::Token1>(signer);

        let stc_amount = 100000;
        let stc = Account::withdraw<STC::STC>(signer, stc_amount);
        let amount_out = {
            let (x, y) = TokenSwap::get_reserves<STC::STC, Token1::Token1>();
            TokenSwapHelper::get_amount_out(stc_amount, x, y)
        };
        let (stc_token, token1_token) = TokenSwap::swap<STC::STC, Token1::Token1>(stc, amount_out, Token::zero<Token1::Token1>(), 0);
        Token::destroy_zero(stc_token);
        Account::deposit_to_self(signer, token1_token);
    }
}

//! new-transaction
//! sender: liquidier
script{
    use 0x1::STC;
    use 0x1::Account;
    use 0x1::Signer;
    use {{admin}}::Token1;
    use {{admin}}::TokenSwap;
    use {{admin}}::LiquidityToken::LiquidityToken;

    // use 0x1::Debug;

    fun main(signer: &signer) {
        let liquidity_balance = Account::balance<LiquidityToken<STC::STC, Token1::Token1>>(Signer::address_of(signer));
        let liquidity = Account::withdraw<LiquidityToken<STC::STC, Token1::Token1>>(signer, liquidity_balance);
        let (stc, token1) = TokenSwap::burn<STC::STC, Token1::Token1>(liquidity);
        Account::deposit_to_self(signer, stc);
        Account::deposit_to_self(signer, token1);

        let (x, y) = TokenSwap::get_reserves<STC::STC, Token1::Token1>();
        assert(x == 0, 111);
        assert(y == 0, 112);
    }
}
// check: EXECUTED
