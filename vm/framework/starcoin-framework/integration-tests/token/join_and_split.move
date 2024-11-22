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
}

// check: EXECUTED

//# run --signers alice
script {
    use std::option;
    use alice::fake_money::{FakeMoney, Self};
    use starcoin_framework::coin;

    fun main(account: signer) {
        fake_money::init(&account, 9);

        let supply = option::destroy_some(coin::supply<FakeMoney>());
        assert!(supply == 0, 8001);
        assert!(coin::is_account_registered<FakeMoney>(@alice), 8002);
        // Create 'Balance<TokenType>' resource under sender account, and init with zero
        // account::do_accept_token<FakeMoney>(&account);
    }
}

// check: EXECUTED

// split and join
//# run --signers alice
script {
    use std::option;
    use std::signer;
    use alice::fake_money::{FakeMoney, Self};
    use starcoin_framework::coin;

    fun main(account: signer) {
        let coin = fake_money::init(&account, 9);
        assert!(coin::value<FakeMoney>(&coin) == 10000, 8002);
        let coin1 = coin::extract<FakeMoney>(coin, 5000);
        let coin2 = coin::extract<FakeMoney>(coin, 5000);
        assert!(coin::value<FakeMoney>(&coin1) == 5000, 8003);
        assert!(coin::value<FakeMoney>(&coin2) == 5000, 8004);
        coin::merge<FakeMoney>(&mut coin1, coin2);
        coin::deposit<FakeMoney>(signer::address_of(&account), coin1);
    }
}

// check: EXECUTED