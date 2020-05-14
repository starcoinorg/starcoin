address 0x0{

module STC {
    use 0x0::Transaction;
    use 0x0::Libra;
    use 0x0::FixedPoint32;

    struct T { }

    public fun initialize() {
        Transaction::assert(Transaction::sender() == 0xA550C18, 0);
        Libra::register_currency<T>(
            FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
            true,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            x"737463" // UTF8-encoded "STC" as a hex string
        );
    }
}
}