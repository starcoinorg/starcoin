address 0x1 {

module Testnet {

    use 0x1::Signer;

    resource struct IsTestnet { }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == 0xA550C18, 0);
        move_to(account, IsTestnet{})
    }

    public fun is_testnet(): bool {
        exists<IsTestnet>(0xA550C18)
    }
}
}
