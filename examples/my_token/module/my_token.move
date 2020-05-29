module MyToken {
     use 0x0::Coin;
     use 0x0::FixedPoint32;
     use 0x0::Account;

     struct T { }

     public fun init() {
         Coin::register_currency<T>(
                     FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
                     1000000, // scaling_factor = 10^6
                     1000,    // fractional_part = 10^3
         );
         Account::add_currency<T>();
     }

     public fun mint(amount: u64) {
        let coin = Coin::mint<T>(amount);
        Account::deposit_to_sender<T>(coin)
     }
}
