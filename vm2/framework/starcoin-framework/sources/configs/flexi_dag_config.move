module starcoin_framework::flexi_dag_config {

    use std::signer;
    use starcoin_framework::system_addresses;
    use starcoin_framework::on_chain_config;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict;
    }

    /// The struct to hold all config data needed for Flexidag.
    struct FlexiDagConfig has copy, drop, store {
        // the height of dag genesis block
        effective_height: u64,
    }

    /// Create a new configuration for flexidag, mainly used in DAO.
    public fun new_flexidag_config(effective_height: u64): FlexiDagConfig {
        FlexiDagConfig {
            effective_height,
        }
    }

    public fun initialize(account: &signer, effective_height: u64) {
        system_addresses::assert_starcoin_framework(account);
        on_chain_config::publish_new_config<FlexiDagConfig>(account, new_flexidag_config(effective_height));
    }

    spec initialize {
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<on_chain_config::Config<FlexiDagConfig>>(signer::address_of(account));
        aborts_if exists<on_chain_config::ModifyConfigCapabilityHolder<FlexiDagConfig>>(signer::address_of(account));
        ensures exists<on_chain_config::Config<FlexiDagConfig>>(signer::address_of(account));
        ensures
            exists<on_chain_config::ModifyConfigCapabilityHolder<FlexiDagConfig>>(
                signer::address_of(account),
            );
    }

    public fun effective_height(account: address): u64 {
        let flexi_dag_config = on_chain_config::get_by_address<FlexiDagConfig>(account);
        flexi_dag_config.effective_height
    }

    spec effective_height {
        include on_chain_config::AbortsIfConfigNotExist<FlexiDagConfig> { addr: account };
    }
}
