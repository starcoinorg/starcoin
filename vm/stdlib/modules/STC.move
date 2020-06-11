address 0x0{

module STC {
    use 0x0::Transaction;
    use 0x0::Coin;
    use 0x0::FixedPoint32;
    use 0x0::Signer;

    struct T { }

    public fun initialize(account: &signer) {
        Transaction::assert(Signer::address_of(account) == 0xA550C18, 0);
        Coin::register_currency<T>(
            account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
        );
    }
}
}