/// This wrapper helps store an on-chain config for the next epoch.
///
/// Once reconfigure with DKG is introduced, every on-chain config `C` should do the following.
/// - Support async update when DKG is enabled. This is typically done by 3 steps below.
///   - Implement `C::set_for_next_epoch()` using `upsert()` function in this module.
///   - Implement `C::on_new_epoch()` using `extract()` function in this module.
///   - Update `0x1::reconfiguration_with_dkg::finish()` to call `C::on_new_epoch()`.
/// - Support sychronous update when DKG is disabled.
///   This is typically done by implementing `C::set()` to update the config resource directly.
///
/// NOTE: on-chain config `0x1::state::ValidatorSet` implemented its own buffer.
module starcoin_framework::config_buffer {
    use std::string::String;
    use starcoin_std::any;
    use starcoin_std::any::Any;
    use starcoin_std::simple_map;
    use starcoin_std::simple_map::SimpleMap;
    use starcoin_std::type_info;
    use starcoin_framework::system_addresses;

    friend starcoin_framework::consensus_config;
    friend starcoin_framework::execution_config;
    friend starcoin_framework::gas_schedule;
    friend starcoin_framework::jwks;
    friend starcoin_framework::jwk_consensus_config;
    friend starcoin_framework::keyless_account;
    friend starcoin_framework::randomness_api_v0_config;
    friend starcoin_framework::randomness_config;
    friend starcoin_framework::randomness_config_seqnum;
    friend starcoin_framework::version;

    /// Config buffer operations failed with permission denied.
    const ESTD_SIGNER_NEEDED: u64 = 1;

    struct PendingConfigs has key {
        configs: SimpleMap<String, Any>,
    }

    public fun initialize(starcoin_framework: &signer) {
        system_addresses::assert_starcoin_framework(starcoin_framework);
        if (!exists<PendingConfigs>(@starcoin_framework)) {
            move_to(starcoin_framework, PendingConfigs {
                configs: simple_map::new(),
            })
        }
    }

    /// Check whether there is a pending config payload for `T`.
    public fun does_exist<T: store>(): bool acquires PendingConfigs {
        if (exists<PendingConfigs>(@starcoin_framework)) {
            let config = borrow_global<PendingConfigs>(@starcoin_framework);
            simple_map::contains_key(&config.configs, &type_info::type_name<T>())
        } else {
            false
        }
    }

    /// Upsert an on-chain config to the buffer for the next epoch.
    ///
    /// Typically used in `X::set_for_next_epoch()` where X is an on-chain config.
    public(friend) fun upsert<T: drop + store>(config: T) acquires PendingConfigs {
        let configs = borrow_global_mut<PendingConfigs>(@starcoin_framework);
        let key = type_info::type_name<T>();
        let value = any::pack(config);
        simple_map::upsert(&mut configs.configs, key, value);
    }

    /// Take the buffered config `T` out (buffer cleared). Abort if the buffer is empty.
    /// Should only be used at the end of a reconfiguration.
    ///
    /// Typically used in `X::on_new_epoch()` where X is an on-chaon config.
    public fun extract<T: store>(): T acquires PendingConfigs {
        let configs = borrow_global_mut<PendingConfigs>(@starcoin_framework);
        let key = type_info::type_name<T>();
        let (_, value_packed) = simple_map::remove(&mut configs.configs, &key);
        any::unpack(value_packed)
    }

    #[test_only]
    struct DummyConfig has drop, store {
        data: u64,
    }

    #[test(fx = @std)]
    fun test_config_buffer_basic(fx: &signer) acquires PendingConfigs {
        initialize(fx);
        // Initially nothing in the buffer.
        assert!(!does_exist<DummyConfig>(), 1);

        // Insert should work.
        upsert(DummyConfig { data: 888 });
        assert!(does_exist<DummyConfig>(), 1);

        // Update and extract should work.
        upsert(DummyConfig { data: 999 });
        assert!(does_exist<DummyConfig>(), 1);
        let config = extract<DummyConfig>();
        assert!(config == DummyConfig { data: 999 }, 1);
        assert!(!does_exist<DummyConfig>(), 1);
    }
}
