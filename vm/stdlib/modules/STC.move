address 0x1{

module STC {
    use 0x1::Coin;
    use 0x1::FixedPoint32;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct STC { }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 0);
        Coin::register_currency<STC>(
            account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
        );
    }

    /// Returns true if `CoinType` is `STC::STC`
    public fun is_stc<CoinType>(): bool {
        Coin::is_currency<CoinType>() &&
            Coin::currency_code<CoinType>() == Coin::currency_code<STC>()
    }
}
}