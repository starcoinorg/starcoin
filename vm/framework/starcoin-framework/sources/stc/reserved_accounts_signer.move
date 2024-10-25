module starcoin_framework::reserved_accounts_signer {

    use starcoin_std::simple_map::{Self, SimpleMap};

    use starcoin_framework::account;
    use starcoin_framework::system_addresses;

    friend starcoin_framework::easy_gas;
    friend starcoin_framework::oracle;

    #[test_only]
    use starcoin_framework::system_addresses::get_starcoin_framework;



    struct SignerResponsbility has key {
        signer_caps: SimpleMap<address, account::SignerCapability>,
    }

    /// Can be called during genesis or by the governance itself.
    /// Stores the signer capability for a given address.
    public fun store_signer_cap(
        starcoin_framework: &signer,
        signer_address: address,
        signer_cap: account::SignerCapability,
    ) acquires SignerResponsbility {
        system_addresses::assert_starcoin_framework(starcoin_framework);
        system_addresses::assert_framework_reserved(signer_address);

        if (!exists<SignerResponsbility>(@starcoin_framework)) {
            move_to(
                starcoin_framework,
                SignerResponsbility { signer_caps: simple_map::create<address, account::SignerCapability>() }
            );
        };

        let signer_caps =
            &mut borrow_global_mut<SignerResponsbility>(@starcoin_framework).signer_caps;
        simple_map::add(signer_caps, signer_address, signer_cap);
    }


    public(friend) fun get_stored_signer(addr: address): signer acquires SignerResponsbility {
        let cap = borrow_global<SignerResponsbility>(system_addresses::get_starcoin_framework());
        account::create_signer_with_capability(simple_map::borrow(&cap.signer_caps, &addr))
    }

    #[test_only]
    public fun initialize_for_test(signer: &signer, cap: account::SignerCapability) acquires SignerResponsbility {
        store_signer_cap(signer, get_starcoin_framework(), cap);
    }

    #[test_only]
    public fun get_genesis_signer_for_test(): signer acquires SignerResponsbility {
        get_stored_signer(@starcoin_framework)
    }
}