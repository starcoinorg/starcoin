//# init -n dev

//# faucet --addr alice --amount 0

//# faucet --addr bob --amount 10000000000000000

//# faucet --addr Genesis --amount 10000000000000000

//# faucet --addr core_resources


//# publish
module bob::fake_money {
    use std::signer;
    use std::string;

    use starcoin_framework::coin;

    struct FakeMoney has key {}

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
        let cap = borrow_global<FakeMoneyCapabilities>(@bob);
        coin::burn(coin, &cap.burn_cap)
    }
}
// check: EXECUTED

//# run --signers bob
script {
    use bob::fake_money::{Self, FakeMoney};
    use starcoin_framework::asset_mapping;

    fun test_create_fake_money_store(account: &signer) {
        fake_money::init(account, 9);
        let fake_money_coin = fake_money::mint(account, 100000000000);
        asset_mapping::create_store_from_coin<FakeMoney>(account, b"bob::fake_money::FakeMoney", fake_money_coin);
    }
}

//# run --signers Genesis
script {
    use bob::fake_money::{FakeMoney};
    use starcoin_framework::coin;
    use starcoin_framework::asset_mapping;

    fun test_create_fake_money_store(account: &signer) {
        asset_mapping::assign_to_account(account, @bob, b"bob::fake_money::FakeMoney", 100000000000);
        assert!(coin::balance<FakeMoney>(@bob) == 100000000000, 10001);
    }
}

//# run --signers core_resources
script {
    use bob::fake_money::{FakeMoney};
    use starcoin_framework::coin;
    use starcoin_framework::asset_mapping;

    fun test_create_fake_money_store(account: &signer) {
        asset_mapping::assign_to_account(account, @bob, b"bob::fake_money::FakeMoney", 100000000000);
        assert!(coin::balance<FakeMoney>(@bob) == 100000000000, 10001);
    }
}

// //# run --signers Genesis
// script {
//     use starcoin_framework::starcoin_coin::STC;
//     use starcoin_framework::coin;
//     use starcoin_framework::asset_mapping;
//
//     fun test_asset_mapping_assign_to_account_with_proof(framework: signer) {
//         assert!(coin::balance<STC>(@alice) == 0, 10001);
//         asset_mapping::assign_to_account(&framework, @alice, b"0x1::STC::STC", 100);
//         assert!(coin::balance<STC>(@alice) == 100, 10002);
//     }
// }
// // check: EXECUTED