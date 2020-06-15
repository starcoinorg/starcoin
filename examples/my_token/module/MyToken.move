module MyToken {
     use 0x1::Coin;
     use 0x1::FixedPoint32;
     use 0x1::Account;

     struct T { }

     public fun init(account: &signer) {
         Coin::register_currency<T>(
                     account,
                     FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
                     1000000, // scaling_factor = 10^6
                     1000,    // fractional_part = 10^3
         );
         Account::add_currency<T>(account);
     }

     public fun mint(account: &signer, amount: u64) {
        let coin = Coin::mint<T>(account, amount);
        Account::deposit_to_sender<T>(account, coin)
     }
}
