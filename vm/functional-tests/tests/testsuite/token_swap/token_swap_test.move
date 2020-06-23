//! account: admin,100_000_000
//! account: liquidier,100_000_000
//! account: exchanger, 10_000_000

//! new-transaction
//! sender: admin
module Math {
    // babylonian method (https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Babylonian_method)
    public fun sqrt(y: u128): u64 {
        if (y < 4) {
            if (y == 0) {
                0u64
            } else {
                1u64
            }
        } else {
            let z = y;
            let x = y / 2 + 1;
            while (x < z) {
                z = x;
                x = (y / x + x) / 2;
            };
            (z as u64)
        }
    }
}
// check: EXECUTED

//! new-transaction
//! sender: admin
module LiquidityToken {
    struct LiquidityToken {}
}
// check: EXECUTED

//! new-transaction
//! sender: admin
module TokenSwapHelper {
    public fun quote(amount_x: u64, reserve_x: u64, reserve_y: u64): u64 {
        assert(amount_x > 0, 400);
        assert(reserve_x > 0 && reserve_y > 0, 410);
        let amount_y = (amount_x as u128) * (reserve_y as u128) / (reserve_x as u128);
        (amount_y as u64)
    }

    public fun get_amount_out(amount_in: u64, reserve_in: u64, reserve_out: u64): u64 {
        assert(amount_in > 0, 400);
        assert(reserve_in > 0 && reserve_out > 0, 410);
        let amount_in_with_fee = (amount_in as u128) * 997;
        let numerator = amount_in_with_fee * (reserve_out as u128);
        let denominator = (reserve_in as u128) * 1000 + amount_in_with_fee;
        ((numerator / denominator) as u64)
    }

    public fun get_amount_in(amount_out: u64, reserve_in: u64, reserve_out: u64): u64 {
        assert(amount_out > 0, 400);
        assert(reserve_in > 0 && reserve_out > 0, 410);
        let numerator = (reserve_in as u128) * (amount_out as u128) * 1000;
        let denominator = (reserve_out - amount_out) * 997;
        ((numerator / (denominator as u128)) as u64) + 1
    }
}
// check: EXECUTED

//! new-transaction
//! sender: admin
module TokenSwap {
    use 0x1::Coin;
    use 0x1::Signer;
    use 0x1::FixedPoint32;
    use {{admin}}::Math;
    use {{admin}}::LiquidityToken::LiquidityToken;
    // Liquidity Token
    // TODO: token should be generic on <TokenX, TokenY>
    // resource struct T {
    // }
    resource struct LiquidityTokenCapability {
        mint: Coin::MintCapability<LiquidityToken>,
        burn: Coin::BurnCapability<LiquidityToken>,
    }

    resource struct TokenPair<TokenX, TokenY> {
        token_x_reserve: Coin::Coin<TokenX>,
        token_y_reserve: Coin::Coin<TokenY>,
        last_block_timestamp: u64,
        last_price_x_cumulative: u128,
        last_price_y_cumulative: u128,
        last_k: u128,
    }


    // resource struct RegisteredSwapPair<TokenX, TokenY> {
    //     holder: address,
    // }


    /// TODO: check X,Y is token, and X,Y is sorted.

    /// Admin methods

    public fun initialize(signer: &signer) {
        assert_admin(signer);

        let exchange_rate = FixedPoint32::create_from_rational(1, 1);
        Coin::register_currency<LiquidityToken>(signer, exchange_rate, 1000000, 1000);

        let mint_capability = Coin::remove_mint_capability<LiquidityToken>(signer);
        let burn_capability = Coin::remove_burn_capability<LiquidityToken>(signer);
        move_to(signer, LiquidityTokenCapability {
            mint: mint_capability,
            burn: burn_capability,
        });
    }

    // for now, only admin can register token pair
    public fun register_swap_pair<TokenX, TokenY>(signer: &signer) {
        assert_admin(signer);
        let token_pair = make_token_pair<TokenX, TokenY>();
        move_to(signer, token_pair);
    }

    fun make_token_pair<X, Y>(): TokenPair<X, Y> {
        // TODO: assert X, Y is coin
        TokenPair<X, Y> {
            token_x_reserve: Coin::zero<X>(),
            token_y_reserve: Coin::zero<Y>(),
            last_block_timestamp: 0,
            last_price_x_cumulative: 0,
            last_price_y_cumulative: 0,
            last_k: 0,
        }
    }

    /// Liquidity Provider's methods

    public fun mint<TokenX, TokenY>(x: Coin::Coin<TokenX>, y: Coin::Coin<TokenY>): Coin::Coin<LiquidityToken>
    acquires TokenPair, LiquidityTokenCapability {
        let total_supply: u128 = Coin::market_cap<LiquidityToken>();
        let x_value = (Coin::value<TokenX>(&x) as u128);
        let y_value = (Coin::value<TokenY>(&y) as u128);

        let liquidity = if (total_supply == 0) {
            // 1000 is the MINIMUM_LIQUIDITY
            Math::sqrt((x_value as u128) * (y_value as u128)) - 1000
        } else {
            let token_pair = borrow_global<TokenPair<TokenX, TokenY>>(admin_address());
            let x_reserve = (Coin::value(&token_pair.token_x_reserve) as u128);
            let y_reserve = (Coin::value(&token_pair.token_y_reserve) as u128);

            let x_liquidity = x_value * total_supply / x_reserve;
            let y_liquidity = y_value * total_supply / y_reserve;

            // use smaller one.
            if (x_liquidity < y_liquidity) {
                (x_liquidity as u64)
            } else {
                (y_liquidity as u64)
            }
        };
        assert(liquidity > 0, 100);

        let token_pair = borrow_global_mut<TokenPair<TokenX, TokenY>>(admin_address());
        Coin::deposit(&mut token_pair.token_x_reserve, x);
        Coin::deposit(&mut token_pair.token_y_reserve, y);

        let liquidity_cap = borrow_global<LiquidityTokenCapability>(admin_address());
        let mint_token = Coin::mint_with_capability(liquidity, &liquidity_cap.mint);
        mint_token
    }

    public fun burn<TokenX, TokenY>(signer: &signer, to_burn: Coin::Coin<LiquidityToken>): (Coin::Coin<TokenX>, Coin::Coin<TokenY>)
    acquires TokenPair, LiquidityTokenCapability {
        let to_burn_value = (Coin::value(&to_burn) as u128);

        let token_pair = borrow_global_mut<TokenPair<TokenX, TokenY>>(admin_address());
        let x_reserve = (Coin::value(&token_pair.token_x_reserve) as u128);
        let y_reserve = (Coin::value(&token_pair.token_y_reserve) as u128);
        let total_supply = Coin::market_cap<LiquidityToken>();

        let x = to_burn_value * x_reserve / total_supply;
        let y = to_burn_value * y_reserve / total_supply;
        assert(x > 0 && y > 0, 101);

        burn_liquidity(to_burn, Signer::address_of(signer));

        let x_token = Coin::withdraw(&mut token_pair.token_x_reserve, (x as u64));
        let y_token = Coin::withdraw(&mut token_pair.token_y_reserve, (y as u64));

        (x_token, y_token)
    }

    fun burn_liquidity(to_burn: Coin::Coin<LiquidityToken>, preburn_address: address)
    acquires LiquidityTokenCapability {
        let liquidity_cap = borrow_global<LiquidityTokenCapability>(admin_address());
        let preburn = Coin::new_preburn_with_capability<LiquidityToken>(&liquidity_cap.burn);
        Coin::preburn_with_resource(to_burn, &mut preburn, preburn_address);
        Coin::burn_with_resource_cap(&mut preburn, preburn_address, &liquidity_cap.burn);
        Coin::destroy_preburn<LiquidityToken>(preburn);
    }

    /// User methods

    public fun get_reserves<TokenX, TokenY>(): (u64, u64) acquires TokenPair {
        let token_pair = borrow_global<TokenPair<TokenX, TokenY>>(admin_address());
        let x_reserve = Coin::value(&token_pair.token_x_reserve);
        let y_reserve = Coin::value(&token_pair.token_y_reserve);
        (x_reserve, y_reserve)
    }

    public fun swap<TokenX, TokenY>(x_in: Coin::Coin<TokenX>, y_out: u64, y_in: Coin::Coin<TokenY>, x_out: u64): (Coin::Coin<TokenX>, Coin::Coin<TokenY>)
    acquires TokenPair {
        let x_in_value = Coin::value(&x_in);
        let y_in_value = Coin::value(&y_in);
        assert(x_in_value > 0 || y_in_value > 0, 400);

        let (x_reserve, y_reserve) = get_reserves<TokenX, TokenY>();

        let token_pair = borrow_global_mut<TokenPair<TokenX, TokenY>>(admin_address());
        Coin::deposit(&mut token_pair.token_x_reserve, x_in);
        Coin::deposit(&mut token_pair.token_y_reserve, y_in);
        let x_swapped = Coin::withdraw(&mut token_pair.token_x_reserve, x_out);
        let y_swapped = Coin::withdraw(&mut token_pair.token_y_reserve, y_out);

        {
            let x_reserve_new = Coin::value(&token_pair.token_x_reserve);
            let y_reserve_new = Coin::value(&token_pair.token_y_reserve);
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
module Coin1 {
    struct Coin1 {}
}
// check: EXECUTED


//! new-transaction
//! sender: admin

// initialize token swap
script {
use {{admin}}::TokenSwap;
fun main(signer: &signer) {
    TokenSwap::initialize(signer);
}
}
// check: EXECUTED

//! new-transaction
//! sender: admin

// register a token pair STC/Coin1
script {
use {{admin}}::TokenSwap;
use {{admin}}::Coin1;
use 0x1::Coin;
use 0x1::STC;
fun main(signer: &signer) {
    Coin::register_currency<Coin1::Coin1>(
        signer,
        0x1::FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
    );
    TokenSwap::register_swap_pair<STC::STC, Coin1::Coin1>(signer);
}
}
// check: EXECUTED

//! new-transaction
//! sender: liquidier
script{
use {{admin}}::Coin1;
use 0x1::Account;
use {{admin}}::LiquidityToken::LiquidityToken;
fun main(signer: &signer) {
    Account::add_currency<Coin1::Coin1>(signer);
    Account::add_currency<LiquidityToken>(signer);
}
}
// check: EXECUTED

//! new-transaction
//! sender: admin
// mint some coin1 to liquidier
script{
use {{admin}}::Coin1;

use 0x1::Account;
fun main(signer: &signer) {
    Account::mint_to_address<Coin1::Coin1>(signer, {{liquidier}}, 100000000);
    assert(Account::balance<Coin1::Coin1>({{liquidier}}) == 100000000, 42);
}
}

//! new-transaction
//! sender: liquidier
script{
    use 0x1::STC;
    use {{admin}}::Coin1;
    use {{admin}}::TokenSwap;
    use 0x1::Account;
    // use 0x1::Debug;

    fun main(signer: &signer) {
        // STC/Coin1 = 1:10
        let stc_amount = 1000000;
        let coin1_amount = 10000000;
        let stc = Account::withdraw_from_sender<STC::STC>(signer, stc_amount);
        let coin1 = Account::withdraw_from_sender<Coin1::Coin1>(signer, coin1_amount);
        let liquidity_token = TokenSwap::mint<STC::STC, Coin1::Coin1>(stc, coin1);
        Account::deposit_to_sender(signer, liquidity_token);

        let (x, y) = TokenSwap::get_reserves<STC::STC, Coin1::Coin1>();
        assert(x == stc_amount, 111);
        assert(y == coin1_amount, 112);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: exchanger
script {
    use 0x1::STC;
    use {{admin}}::Coin1;
    use {{admin}}::TokenSwap;
    use {{admin}}::TokenSwapHelper;
    use 0x1::Account;
    use 0x1::Coin;
    fun main(signer: &signer) {
        Account::add_currency<Coin1::Coin1>(signer);

        let stc_amount = 100000;
        let stc = Account::withdraw_from_sender<STC::STC>(signer, stc_amount);
        let amount_out = {
            let (x, y) = TokenSwap::get_reserves<STC::STC, Coin1::Coin1>();
            TokenSwapHelper::get_amount_out(stc_amount, x, y)
        };
        let (stc_token, coin1_token) = TokenSwap::swap<STC::STC, Coin1::Coin1>(stc, amount_out, Coin::zero<Coin1::Coin1>(), 0);
        Coin::destroy_zero(stc_token);
        Account::deposit_to_sender(signer, coin1_token);
    }
}

//! new-transaction
//! sender: liquidier
script{
    use 0x1::STC;
    use 0x1::Account;
    use 0x1::Signer;
    use {{admin}}::Coin1;
    use {{admin}}::TokenSwap;
    use {{admin}}::LiquidityToken::LiquidityToken;


    // use 0x1::Debug;

    fun main(signer: &signer) {
        let liquidity_balance = Account::balance<LiquidityToken>(Signer::address_of(signer));
        let liquidity = Account::withdraw_from_sender<LiquidityToken>(signer, liquidity_balance);
        let (stc, coin1) = TokenSwap::burn<STC::STC, Coin1::Coin1>(signer, liquidity);
        Account::deposit_to_sender(signer, stc);
        Account::deposit_to_sender(signer, coin1);

        let (x, y) = TokenSwap::get_reserves<STC::STC, Coin1::Coin1>();
        assert(x == 0, 111);
        assert(y == 0, 112);
    }
}
// check: EXECUTED
