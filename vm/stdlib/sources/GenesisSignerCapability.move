module Std::GenesisSignerCapability {
    use Std::Account;
    use Std::CoreAddresses;
    use Std::Errors;

    friend Std::NFT;
    friend Std::Oracle;
    friend Std::Genesis;
    friend Std::StdlibUpgradeScripts;


    const ENOT_GENESIS_ACCOUNT: u64 = 11;

    struct GenesisSignerCapability has key {
        cap: Account::SignerCapability,
    }

    public(friend) fun initialize(signer:&signer, cap: Account::SignerCapability) {
        CoreAddresses::assert_genesis_address(signer);
        assert!(Account::signer_address(&cap) == CoreAddresses::GENESIS_ADDRESS(), Errors::invalid_argument(ENOT_GENESIS_ACCOUNT));
        move_to(signer, GenesisSignerCapability{cap});
    }

    public(friend) fun get_genesis_signer(): signer acquires GenesisSignerCapability {
        let cap = borrow_global<GenesisSignerCapability>(CoreAddresses::GENESIS_ADDRESS());
        Account::create_signer_with_cap(&cap.cap)
    }
}