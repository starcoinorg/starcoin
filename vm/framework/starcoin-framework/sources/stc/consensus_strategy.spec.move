/// The module provides the information of current consensus strategy.
spec starcoin_framework::consensus_strategy {
    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    spec initialize(account: &signer, consensus_strategy: u8) {
        // aborts_if !Timestamp::is_genesis();
        use std::signer;
        use starcoin_framework::on_chain_config;

        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<on_chain_config::Config<ConsensusStrategy>>(signer::address_of(account));
        aborts_if exists<on_chain_config::ModifyConfigCapabilityHolder<ConsensusStrategy>>(signer::address_of(account));
        ensures exists<on_chain_config::Config<ConsensusStrategy>>(signer::address_of(account));
    }

    spec get {
        aborts_if !exists<on_chain_config::Config<ConsensusStrategy>>(system_addresses::get_starcoin_framework());
    }
}
