//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# publish
module alice::fake_money {
    use std::signer;
    use std::string;

    use starcoin_framework::coin;

    struct FakeMoney {}

    struct FakeMoneyCapabilities has key {
        burn_cap: coin::BurnCapability<FakeMoney>,
        freeze_cap: coin::FreezeCapability<FakeMoney>,
        mint_cap: coin::MintCapability<FakeMoney>,
    }

    public fun init(account: &signer, decimal: u8) {
        let (
            burn_cap,
            freeze_cap,
            mint_cap
        ) = coin::initialize<FakeMoney>(
            account,
            string::utf8(b"FakeMoney"),
            string::utf8(b"FakeMoney"),
            decimal,
            true,
        );
        coin::register<FakeMoney>(account);
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        })
    }

    public fun mint(account: &signer, amount: u64): coin::Coin<FakeMoney> acquires FakeMoneyCapabilities {
        let cap = borrow_global<FakeMoneyCapabilities>(signer::address_of(account));
        coin::mint(amount, &cap.mint_cap)
    }

    public fun burn(coin: coin::Coin<FakeMoney>) acquires FakeMoneyCapabilities {
        let cap = borrow_global<FakeMoneyCapabilities>(@alice);
        coin::burn(coin, &cap.burn_cap)
    }
}

//# run --signers alice
script {
    use std::option;
    use alice::fake_money::{Self, FakeMoney};
    use starcoin_framework::coin;

    fun init_and_register(account: signer) {
        fake_money::init(&account, 9);

        let market_cap = coin::supply<FakeMoney>();
        assert!(option::destroy_some(market_cap) == 0, 8001);
        assert!(coin::is_account_registered<FakeMoney>(@alice), 8002);
        // Create 'Balance<TokenType>' resource under sender account, and init with zero
        coin::register<FakeMoney>(&account);
    }
}


//# run --signers alice
script {
    use std::option;
    use std::signer;
    use starcoin_framework::coin;
    use alice::fake_money::{Self, FakeMoney};

    fun test_supply(account: signer) {
        // mint 100 coins and check that the market cap increases appropriately
        let old_market_supply = option::destroy_some(coin::supply<FakeMoney>());
        let coin = fake_money::mint(&account, 10000);
        assert!(coin::value<FakeMoney>(&coin) == 10000, 8002);
        assert!(option::destroy_some(coin::supply<FakeMoney>()) == old_market_supply + 10000, 8003);
        coin::deposit<FakeMoney>(signer::address_of(&account), coin);
    }
}

// //# run --signers alice
// script {
//     use starcoin_framework::coin;
//     use alice::fake_money::{FakeMoney};
//
//     fun test_withdraw_and_burn(account: signer) {
//         // Token::add_burn_capability<FakeMoney>(&account, cap);
//     }
// }


//# run --signers alice
script {
    use std::option;
    use std::string;
    use starcoin_framework::coin;
    use alice::fake_money::{FakeMoney, Self};
    use starcoin_std::debug;

    fun test_withdraw_and_burn(account: signer) {
        let supply = option::destroy_some(coin::supply<FakeMoney>());

        assert!(supply == 10000, 8004);

        debug::print(&string::utf8(b"test_withdraw_and_burn | 1"));
        let token = coin::withdraw<FakeMoney>(&account, 10000);
        debug::print(&string::utf8(b"test_withdraw_and_burn | 2"));
        let t1 = coin::extract<FakeMoney>(&mut token, 100);
        debug::print(&string::utf8(b"test_withdraw_and_burn | 3"));
        let t2 = coin::extract<FakeMoney>(&mut token, 10000); // amount is not enough
        fake_money::burn(token);
        fake_money::burn(t1);
        fake_money::burn(t2);
    }
}


//# run --signers alice
script {
    use std::option;
    use alice::fake_money::{FakeMoney, Self};
    use starcoin_framework::coin;

    fun test_mint_and_burn(account: signer) {
        let old_market_cap = option::destroy_some(coin::supply<FakeMoney>());
        let amount = 100;
        let coin = fake_money::mint(&account, amount);
        assert!(coin::value<FakeMoney>(&coin) == amount, 8008);
        assert!(option::destroy_some(coin::supply<FakeMoney>()) == (old_market_cap + (amount as u128)), 8009);
        fake_money::burn(coin);
    }
}

//# run --signers alice
script {
    use alice::fake_money::{FakeMoney};
    use starcoin_framework::coin;

    fun test_withdraw_and_burn(account: signer) {
        let zero = coin::withdraw<FakeMoney>(&account, 0);
        coin::destroy_zero<FakeMoney>(zero);
        let token = coin::withdraw<FakeMoney>(&account, 10); //EDESTROY_TOKEN_NON_ZERO
        coin::destroy_zero<FakeMoney>(token);
    }
}

// //# run --signers alice
// script {
//     use starcoin_framework::Token;
//     use alice::fake_money::{FakeMoney};
//     use starcoin_framework::coin;
//
//     fun test_withdraw_and_burn(account: signer) {
//         let burn_cap = coin::remove_burn_capability<FakeMoney>(&account);
//         coin::destroy_burn_capability<FakeMoney>(burn_cap);
//         let mint_cap = Token::remove_mint_capability<FakeMoney>(&account);
//         Token::destroy_mint_capability<FakeMoney>(mint_cap);
//     }
// }
