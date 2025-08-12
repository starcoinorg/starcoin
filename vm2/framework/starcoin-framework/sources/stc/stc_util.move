module starcoin_framework::stc_util {

    use std::string;
    use starcoin_framework::chain_id;
    use starcoin_std::type_info;

    const CHAIN_ID_MAIN: u8 = 1;
    const CHAIN_ID_VEGA: u8 = 2;
    const CHAIN_ID_BARNARD: u8 = 251;
    const CHAIN_ID_PROXIMA: u8 = 252;
    const CHAIN_ID_HALLEY: u8 = 253;
    const CHAIN_ID_DEV: u8 = 254;
    const CHAIN_ID_TEST: u8 = 255;

    #[view]
    public fun is_stc<Coin>(): bool {
        type_info::type_name<Coin>() == string::utf8(b"0x00000000000000000000000000000001::starcoin_coin::STC")
    }

    #[view]
    public fun token_issuer<Coin>(): address {
        type_info::account_address(&type_info::type_of<Coin>())
    }

    #[view]
    public fun is_net_dev(): bool {
        chain_id::get() == CHAIN_ID_DEV
    }

    #[view]
    public fun is_net_test(): bool {
        chain_id::get() == CHAIN_ID_TEST
    }

    #[view]
    public fun is_net_halley(): bool {
        chain_id::get() == CHAIN_ID_HALLEY
    }

    #[view]
    public fun is_net_barnard(): bool {
        chain_id::get() == CHAIN_ID_BARNARD
    }


    #[view]
    public fun is_net_main(): bool {
        chain_id::get() == CHAIN_ID_MAIN
    }

    #[view]
    public fun is_net_vega(): bool {
        chain_id::get() == CHAIN_ID_VEGA
    }
}
