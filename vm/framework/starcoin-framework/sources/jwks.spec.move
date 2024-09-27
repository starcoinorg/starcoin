spec starcoin_framework::jwks {
    spec on_new_epoch(framework: &signer) {
        requires @starcoin_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<SupportedOIDCProviders>;
        aborts_if false;
    }
}
