module starcoin_framework::dao_features_proposal {

    use std::error;
    use std::features;
    use std::signer;
    use std::vector;

    use starcoin_framework::create_signer::create_signer;
    use starcoin_framework::dao;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::system_addresses;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    struct FeaturesUpdate has copy, drop, store {
        enable: vector<u64>,
        disable: vector<u64>,
    }

    const E_NOT_AUTHORIZED: u64 = 1;
    const E_NOT_ANY_FLAGS: u64 = 2;

    /// Entrypoint for the proposal.
    public entry fun propose(
        signer: &signer,
        enable: vector<u64>,
        disable: vector<u64>,
        exec_delay: u64,
    ) {
        assert!(
            !vector::is_empty(&enable) || !vector::is_empty(&disable),
            error::invalid_argument(E_NOT_ANY_FLAGS)
        );
        let action = FeaturesUpdate {
            enable,
            disable,
        };
        dao::propose<STC, FeaturesUpdate>(signer, action, exec_delay);
    }

    /// Once the proposal is agreed, anyone can call the method to make the proposal happen.
    public entry fun execute(proposal_adderss: address, proposal_id: u64) {
        let FeaturesUpdate {
            enable,
            disable,
        } = dao::extract_proposal_action<STC, FeaturesUpdate>(
            proposal_adderss,
            proposal_id
        );
        let starcoin_framework = &create_signer(system_addresses::get_starcoin_framework());
        features::change_feature_flags_for_next_epoch(starcoin_framework, enable, disable);
        features::on_new_epoch(starcoin_framework);
    }


    public entry fun execute_urgent(core_resource: &signer, enable: vector<u64>, disable: vector<u64>) {
        assert!(signer::address_of(core_resource) == @core_resources, error::unauthenticated(E_NOT_AUTHORIZED));
        let framework = &create_signer(@starcoin_framework);
        features::change_feature_flags_for_next_epoch(framework, enable, disable);
        features::on_new_epoch(framework);
    }
}