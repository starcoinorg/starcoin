spec starcoin_framework::jwk_consensus_config {
    spec on_new_epoch(framework: &signer) {
        requires @starcoin_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<JWKConsensusConfig>;
        aborts_if false;
    }
}
