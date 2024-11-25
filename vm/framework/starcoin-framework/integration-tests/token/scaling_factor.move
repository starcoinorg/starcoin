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

//# run --signers alice
script {
    use alice::fake_money;

    fun main(account: signer) {
        fake_money::init(&account, 39); // ECOIN_COIN_DECIMAL_TOO_LARGE
    }
}
// check: "Keep(ABORTED { code: 65565"

//# run --signers alice
script {
    use alice::fake_money;

    fun main(account: signer) {
        fake_money::init(&account, 3);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::coin;
    use alice::fake_money::{FakeMoney};
    use starcoin_std::debug;

    fun check_decimal(_account: signer) {
        let decimal = coin::decimals<FakeMoney>();
        debug::print(&std::string::utf8(b"scaling_factor.move - check_decimal"));
        debug::print(&decimal);
        assert!(decimal == 3, 101);
    }
}
// check: EXECUTED