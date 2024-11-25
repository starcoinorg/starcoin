/// dao_upgrade_module_proposal is a proposal moudle used to upgrade contract codes under a token.
module starcoin_framework::dao_upgrade_module_proposal {

    use std::error;
    use std::signer;

    use starcoin_framework::dao;
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::stc_util;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    const ERR_UNABLE_TO_UPGRADE: u64 = 400;
    const ERR_NOT_AUTHORIZED: u64 = 401;
    const ERR_ADDRESS_MISSMATCH: u64 = 402;

    /// A wrapper of `stc_transaction_package_validation::UpgradePlanCapability`.
    struct UpgradeModuleCapability<phantom TokenT> has key {
        cap: stc_transaction_package_validation::UpgradePlanCapability,
    }

    struct UpgradeModuleV2 has copy, drop, store {
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
        enforced: bool,
    }

    /// If this goverment can upgrade module, call this to register capability.
    public fun plugin<TokenT>(
        signer: &signer,
        cap: stc_transaction_package_validation::UpgradePlanCapability,
    ) {
        let token_issuer = stc_util::token_issuer<TokenT>();
        assert!(signer::address_of(signer) == token_issuer, error::unauthenticated(ERR_NOT_AUTHORIZED));
        move_to(signer, UpgradeModuleCapability<TokenT> { cap })
    }

    spec plugin {
        use std::signer;

        pragma aborts_if_is_partial = false;

        let sender = signer::address_of(signer);
        aborts_if sender != @0x2;
        aborts_if exists<UpgradeModuleCapability<TokenT>>(sender);
    }

    spec schema AbortIfUnableUpgrade<TokenT> {
        module_address: address;
        let token_issuer = @0x2;
        aborts_if !exists<UpgradeModuleCapability<TokenT>>(token_issuer);
        let cap = global<UpgradeModuleCapability<TokenT>>(token_issuer).cap;
        aborts_if stc_transaction_package_validation::account_address(cap) != module_address;
    }

    public fun propose_module_upgrade_v2<TokenT>(
        signer: &signer,
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
        exec_delay: u64,
        enforced: bool,
    ) acquires UpgradeModuleCapability {
        let cap = borrow_global<UpgradeModuleCapability<TokenT>>(stc_util::token_issuer<TokenT>());
        let account_address = stc_transaction_package_validation::account_address(&cap.cap);
        assert!(account_address == module_address, error::permission_denied(ERR_ADDRESS_MISSMATCH));
        dao::propose<TokenT, UpgradeModuleV2>(
            signer,
            UpgradeModuleV2 { module_address, package_hash, version, enforced },
            exec_delay,
        );
    }

    spec propose_module_upgrade_v2 {
        pragma aborts_if_is_partial = true;
        include AbortIfUnableUpgrade<TokenT>;
    }

    /// Once the proposal is agreed, anyone can call this method to generate the upgrading plan.
    public fun submit_module_upgrade_plan<TokenT>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires UpgradeModuleCapability {
        let UpgradeModuleV2 {
            module_address, package_hash, version, enforced
        } = dao::extract_proposal_action<
            TokenT,
            UpgradeModuleV2,
        >(proposer_address, proposal_id);
        let cap = borrow_global<UpgradeModuleCapability<TokenT>>(stc_util::token_issuer<TokenT>());
        let account_address = stc_transaction_package_validation::account_address(&cap.cap);
        assert!(account_address == module_address, error::permission_denied(ERR_ADDRESS_MISSMATCH));
        stc_transaction_package_validation::submit_upgrade_plan_with_cap_v2(
            &cap.cap,
            package_hash,
            version,
            enforced,
        );
    }
    spec submit_module_upgrade_plan {
        use std::option;

        let expected_states = vec<u8>(6);
        include dao::CheckProposalStates<TokenT, UpgradeModuleV2> { expected_states };
        let proposal = global<dao::Proposal<TokenT, UpgradeModuleV2>>(proposer_address);
        aborts_if option::is_none(proposal.action);
        let action = proposal.action.vec[0];
        include AbortIfUnableUpgrade<TokenT> { module_address: action.module_address };
    }
}