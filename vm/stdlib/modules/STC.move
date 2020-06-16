address 0x1{

module STC {
    use 0x1::Coin;
    use 0x1::FixedPoint32;
    use 0x1::Signer;

    struct STC { }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == 0xA550C18, 0);
        Coin::register_currency<STC>(
            account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
        );
    }
}
}