/// Define the GovernanceProposal that will be used as part of on-chain governance by StarcoinGovernance.
///
/// This is separate from the StarcoinGovernance module to avoid circular dependency between StarcoinGovernance and Stake.
module starcoin_framework::governance_proposal {
    friend starcoin_framework::starcoin_governance;

    struct GovernanceProposal has store, drop {}

    /// Create and return a GovernanceProposal resource. Can only be called by StarcoinGovernance
    public(friend) fun create_proposal(): GovernanceProposal {
        GovernanceProposal {}
    }

    /// Useful for StarcoinGovernance to create an empty proposal as proof.
    public(friend) fun create_empty_proposal(): GovernanceProposal {
        create_proposal()
    }

    #[test_only]
    public fun create_test_proposal(): GovernanceProposal {
        create_empty_proposal()
    }
}
