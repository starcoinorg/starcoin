address 0x1 {
module UpgradeModuleDaoProposal {
    use 0x1::PackageTxnManager;
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Option;
    use 0x1::Dao;
    use 0x1::Block;
    use 0x1::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    const ERR_UNABLE_TO_UPGRADE: u64 = 400;
    const ERR_NOT_AUTHORIZED: u64 = 401;
    const ERR_ADDRESS_MISSMATCH: u64 = 402;

    resource struct UpgradeModuleCapability<TokenT> {
        cap: PackageTxnManager::UpgradePlanCapability,
    }

    // const UPGRADE_DELAY: u64 = 200;
    struct UpgradeModule {
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
    }

    /// If this govverment can upgrade module, call this to register capability.
    public fun plugin<TokenT>(
        signer: &signer,
        cap: PackageTxnManager::UpgradePlanCapability,
    ) {
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        move_to(signer, UpgradeModuleCapability<TokenT> { cap })
    }

    spec fun plugin {
        pragma aborts_if_is_partial = false;

        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if exists<UpgradeModuleCapability<TokenT>>(sender);
    }

    spec schema AbortIfUnableUpgrade<TokenT> {
        module_address: address;
        let token_issuer = Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<UpgradeModuleCapability<TokenT>>(token_issuer);
        let cap = global<UpgradeModuleCapability<TokenT>>(token_issuer).cap;
        aborts_if PackageTxnManager::account_address(cap) != module_address;
    }

    /// propose a module upgrade, called by proposer.
    public fun propose_module_upgrade<TokenT: copyable>(
        signer: &signer,
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
        exec_delay: u64,
    ) acquires UpgradeModuleCapability {
        let cap = borrow_global<UpgradeModuleCapability<TokenT>>(Token::token_address<TokenT>());
        let account_address = PackageTxnManager::account_address(&cap.cap);
        assert(account_address == module_address, Errors::requires_capability(ERR_ADDRESS_MISSMATCH));
        Dao::propose<TokenT, UpgradeModule>(
            signer,
            UpgradeModule { module_address, package_hash, version },
            exec_delay,
        );
    }

    spec fun propose_module_upgrade {
        pragma aborts_if_is_partial = true;
        include AbortIfUnableUpgrade<TokenT>;
    }

    public fun submit_module_upgrade_plan<TokenT: copyable>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires UpgradeModuleCapability {
        let UpgradeModule { module_address, package_hash, version } = Dao::extract_proposal_action<
            TokenT,
            UpgradeModule,
        >(proposer_address, proposal_id);
        let cap = borrow_global<UpgradeModuleCapability<TokenT>>(Token::token_address<TokenT>());
        let account_address = PackageTxnManager::account_address(&cap.cap);
        assert(account_address == module_address, Errors::requires_capability(ERR_ADDRESS_MISSMATCH));
        PackageTxnManager::submit_upgrade_plan_with_cap(
            &cap.cap,
            package_hash,
            version,
            Block::get_current_block_number(),
        );
    }
    spec fun submit_module_upgrade_plan {
        let expected_states = singleton_vector(6);
        include Dao::CheckProposalStates<TokenT, UpgradeModule>{expected_states};
        let proposal = global<Dao::Proposal<TokenT, UpgradeModule>>(proposer_address);
        aborts_if Option::spec_is_none(proposal.action);
        let action = proposal.action.vec[0];
        include AbortIfUnableUpgrade<TokenT>{module_address: action.module_address};
    }
}
}