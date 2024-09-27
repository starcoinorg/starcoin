spec starcoin_framework::randomness_config_seqnum {
    spec on_new_epoch(framework: &signer) {
        requires @starcoin_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<RandomnessConfigSeqNum>;
        aborts_if false;
    }
}
