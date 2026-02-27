//# init -n dev


//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol

//# publish
module alice::batch_test_helper {
    use std::signer;
    use std::string;

    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::STC;

    struct FakeMoney has store {}

    struct FakeMoneyCapabilities has key {
        burn_cap: coin::BurnCapability<FakeMoney>,
        freeze_cap: coin::FreezeCapability<FakeMoney>,
        mint_cap: coin::MintCapability<FakeMoney>,
    }

    struct StcSnapshot has key {
        bob_balance: u64,
        carol_balance: u64,
    }

    public fun init_fake_money(account: &signer) {
        let (
            burn_cap,
            freeze_cap,
            mint_cap
        ) = coin::initialize<FakeMoney>(
            account,
            string::utf8(b"BatchFakeMoney"),
            string::utf8(b"BFM"),
            9,
            true,
        );
        coin::register<FakeMoney>(account);
        move_to(account, FakeMoneyCapabilities {
            burn_cap,
            freeze_cap,
            mint_cap,
        })
    }

    public fun mint_fake_money(account: &signer, amount: u64): coin::Coin<FakeMoney> acquires FakeMoneyCapabilities {
        let caps = borrow_global<FakeMoneyCapabilities>(signer::address_of(account));
        coin::mint(amount, &caps.mint_cap)
    }

    public fun save_stc_snapshot(account: &signer) acquires StcSnapshot {
        let account_addr = signer::address_of(account);
        let bob_balance = coin::balance<STC>(@bob);
        let carol_balance = coin::balance<STC>(@carol);

        if (exists<StcSnapshot>(account_addr)) {
            let snapshot = borrow_global_mut<StcSnapshot>(account_addr);
            snapshot.bob_balance = bob_balance;
            snapshot.carol_balance = carol_balance;
        } else {
            move_to(account, StcSnapshot { bob_balance, carol_balance });
        }
    }

    public fun assert_stc_snapshot(account: &signer) acquires StcSnapshot {
        let snapshot = borrow_global<StcSnapshot>(signer::address_of(account));
        assert!(coin::balance<STC>(@bob) == snapshot.bob_balance, 2001);
        assert!(coin::balance<STC>(@carol) == snapshot.carol_balance, 2002);
    }
}


//# run --signers alice --args @bob --args x"" --args 100u128
script {
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer, payee: address, payee_auth_key: vector<u8>, amount: u128) {
        transfer_scripts::peer_to_peer<STC>(&account, payee, payee_auth_key, amount);
    }
}

//# run --signers alice --args @bob --args 100u128
script {
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer, payee: address, amount: u128) {
        transfer_scripts::peer_to_peer_v2<STC>(&account, payee, amount);
    }
}

//# run --signers alice
script {
    use starcoin_framework::coin;
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer) {
        let bob_before = coin::balance<STC>(@bob);
        let carol_before = coin::balance<STC>(@carol);

        transfer_scripts::batch_peer_to_peer_v2<STC>(
            &account,
            vector[@bob, @carol],
            vector[100u128, 200u128],
        );

        assert!(coin::balance<STC>(@bob) == bob_before + 100, 1001);
        assert!(coin::balance<STC>(@carol) == carol_before + 200, 1002);
    }
}

//# run --signers alice
script {
    use alice::batch_test_helper::{Self, FakeMoney};
    use starcoin_framework::coin;

    fun main(account: signer) {
        batch_test_helper::init_fake_money(&account);
        coin::deposit<FakeMoney>(@alice, batch_test_helper::mint_fake_money(&account, 1000));
    }
}

//# run --signers bob
script {
    use alice::batch_test_helper::FakeMoney;
    use starcoin_framework::coin;

    fun main(account: signer) {
        coin::register<FakeMoney>(&account);
    }
}

//# run --signers carol
script {
    use alice::batch_test_helper::FakeMoney;
    use starcoin_framework::coin;

    fun main(account: signer) {
        coin::register<FakeMoney>(&account);
    }
}

//# run --signers alice
script {
    use alice::batch_test_helper::FakeMoney;
    use starcoin_framework::coin;

    fun main(account: signer) {
        let bob_before = coin::balance<FakeMoney>(@bob);
        let carol_before = coin::balance<FakeMoney>(@carol);

        coin::transfer<FakeMoney>(&account, @bob, 111);
        coin::transfer<FakeMoney>(&account, @carol, 222);

        assert!(coin::balance<FakeMoney>(@bob) == bob_before + 111, 1101);
        assert!(coin::balance<FakeMoney>(@carol) == carol_before + 222, 1102);
        assert!(coin::balance<FakeMoney>(@alice) == 667, 1103);
    }
}

//# run --signers alice
script {
    use alice::batch_test_helper;

    fun main(account: signer) {
        batch_test_helper::save_stc_snapshot(&account);
    }
}

//# run --signers alice
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::transfer_scripts;

    fun main(account: signer) {
        transfer_scripts::batch_peer_to_peer_v2<STC>(
            &account,
            vector[@bob, @carol],
            vector[1u128],
        );
    }
}

//# run --signers alice
script {
    use alice::batch_test_helper;

    fun main(account: signer) {
        batch_test_helper::assert_stc_snapshot(&account);
    }
}
