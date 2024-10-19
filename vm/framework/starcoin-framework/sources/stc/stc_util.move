module starcoin_framework::stc_util {

    use std::string;
    use starcoin_std::type_info;

    #[view]
    public fun is_stc<Coin>(): bool {
        type_info::type_name<Coin>() == string::utf8(b"0x1::starcoin_coin::STC")
    }
}
