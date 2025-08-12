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
// check: EXECUTED

//# publish
module bob::HideToken {
    use alice::fake_money::FakeMoney;

    use starcoin_framework::coin;

    struct Collection has key, store {
        t: coin::Coin<FakeMoney>,
    }

    public fun hide(account: &signer, coin: coin::Coin<FakeMoney>) {
        let b = Collection { t: coin };
        move_to<Collection>(account, b);
    }
}


//# run --signers alice
script {
    use std::option;
    use alice::fake_money::{Self, FakeMoney};
    use starcoin_framework::coin;

    fun main(account: signer) {
        fake_money::init(&account, 9);

        let market_cap = option::destroy_some(coin::supply<FakeMoney>());
        assert!(market_cap == 0, 8001);
        assert!(coin::is_account_registered<FakeMoney>(@alice), 8002);
        // Create 'Balance<TokenType>' resource under sender account, and init with zero
        coin::register<FakeMoney>(&account);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use std::option;
    use std::signer;
    use starcoin_framework::coin;
    use alice::fake_money::{FakeMoney, Self};

    fun main(account: signer) {
        // mint 100 coins and check that the market cap increases appropriately
        let old_market_cap = option::destroy_some(coin::supply<FakeMoney>());
        let coin = fake_money::mint(&account, 10000);
        assert!(coin::value<FakeMoney>(&coin) == 10000, 8002);
        assert!(option::destroy_some(coin::supply<FakeMoney>()) == old_market_cap + 10000, 8003);
        coin::deposit<FakeMoney>(signer::address_of(&account), coin);
    }
}
// check: EXECUTED

//# run --signers bob
script {
    use alice::fake_money::FakeMoney;
    use starcoin_framework::coin;

    fun main(account: signer) {
        coin::register<FakeMoney>(&account);
    }
}


//# run --signers alice
script {
    use alice::fake_money::FakeMoney;
    use starcoin_framework::coin;

    fun main(account: signer) {
        coin::transfer<FakeMoney>(&account, @bob, 10);
    }
}

//# run --signers bob
script {
    use alice::fake_money::FakeMoney;
    use bob::HideToken;
    use starcoin_framework::coin;

    fun main(account: signer) {
        let token = coin::withdraw<FakeMoney>(&account, 10);
        HideToken::hide(&account, token);
    }
}