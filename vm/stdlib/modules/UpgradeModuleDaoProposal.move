address 0x1 {
module UpgradeModuleDaoProposal {
    use 0x1::PackageTxnManager;
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::Option;
    use 0x1::Dao;
    use 0x1::Block;
    use 0x1::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;
    const ERR_UNABLE_TO_UPGRADE: u64 = 400;

    resource struct UpgradeModuleCapabilities<TokenT> {
        caps: vector<WrappedUpgradePlanCapability>,
    }

    resource struct WrappedUpgradePlanCapability {
        cap: PackageTxnManager::UpgradePlanCapability,
    }

    // const UPGRADE_DELAY: u64 = 200;
    struct UpgradeModule {
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
    }

    public fun plugin<TokenT>(signer: &signer) {
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let caps = UpgradeModuleCapabilities<TokenT> { caps: Vector::empty() };
        move_to(signer, caps)
    }
    spec fun plugin {
        pragma aborts_if_is_partial = false;
        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if exists<UpgradeModuleCapabilities<TokenT>>(sender);
        ensures exists<UpgradeModuleCapabilities<TokenT>>(sender);
        ensures len(global<UpgradeModuleCapabilities<TokenT>>(sender).caps) == 0;
    }

    /// If this govverment can upgrade module, call this to register capability.
    public fun delegate_module_upgrade_capability<TokenT>(
        signer: &signer,
        cap: PackageTxnManager::UpgradePlanCapability,
    ) acquires UpgradeModuleCapabilities {
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let caps = borrow_global_mut<UpgradeModuleCapabilities<TokenT>>(token_issuer);
        // TODO: should check duplicate cap?
        // for now, only one cap exists for a module address.
        Vector::push_back(&mut caps.caps, WrappedUpgradePlanCapability { cap });
    }

    spec fun delegate_module_upgrade_capability {
        pragma aborts_if_is_partial = false;

        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<UpgradeModuleCapabilities<TokenT>>(sender);
        ensures len(global<UpgradeModuleCapabilities<TokenT>>(sender).caps) == len(old(global<UpgradeModuleCapabilities<TokenT>>(sender).caps)) + 1;
    }

    /// check whether this gov has the ability to upgrade module in `moudle_address`.
    public fun able_to_upgrade<TokenT>(module_address: address): bool
    acquires UpgradeModuleCapabilities {
        let pos = find_module_upgrade_cap<TokenT>(module_address);
        Option::is_some(&pos)
    }

    spec fun able_to_upgrade {
        pragma aborts_if_is_partial = false;
        let token_address = Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<UpgradeModuleCapabilities<TokenT>>(token_address);
    }

    spec schema AbortIfUnableUpgrade<TokenT> {
        module_address: address;
        let token_issuer = Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<UpgradeModuleCapabilities<TokenT>>(token_issuer);
        let caps = global<UpgradeModuleCapabilities<TokenT>>(token_issuer).caps;
        aborts_if (forall cap in caps: PackageTxnManager::account_address(cap.cap) != module_address);
    }

    /// propose a module upgrade, called by proposer.
    public fun propose_module_upgrade<TokenT: copyable>(
        signer: &signer,
        module_address: address,
        package_hash: vector<u8>,
        version: u64,
        exec_delay: u64,
    ) acquires UpgradeModuleCapabilities {
        assert(able_to_upgrade<TokenT>(module_address), Errors::requires_capability(ERR_UNABLE_TO_UPGRADE));
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
    ) acquires UpgradeModuleCapabilities {
        let UpgradeModule { module_address, package_hash, version } = Dao::extract_proposal_action<
            TokenT,
            UpgradeModule,
        >(proposer_address, proposal_id);
        let pos = find_module_upgrade_cap<TokenT>(module_address);
        assert(Option::is_some(&pos), Errors::requires_capability(ERR_UNABLE_TO_UPGRADE));
        spec {
            assert Option::spec_is_some(pos);
        };
        let pos = Option::extract(&mut pos);
        let caps = borrow_global<UpgradeModuleCapabilities<TokenT>>(Token::token_address<TokenT>());
        spec {
            assert len(caps.caps) > 0;
            // todo: figure out why this fail.
            // assert pos < len(caps.caps);
        };
        let cap = Vector::borrow(&caps.caps, pos);
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

    fun find_module_upgrade_cap<TokenT>(module_address: address): Option::Option<u64>
    acquires UpgradeModuleCapabilities {
        let token_issuer = Token::token_address<TokenT>();
        let caps = borrow_global<UpgradeModuleCapabilities<TokenT>>(token_issuer);
        let cap_len = Vector::length(&caps.caps);
        let i = 0;
        while (i < cap_len){
            spec {
                assert i < cap_len;
            };
            let cap = Vector::borrow(&caps.caps, i);
            let account_address = PackageTxnManager::account_address(&cap.cap);
            if (account_address == module_address) {
                return Option::some(i)
            };
            i = i + 1;
        };
        Option::none<u64>()
    }

    spec fun find_module_upgrade_cap {
        pragma aborts_if_is_partial = false;
        let token_address = Token::SPEC_TOKEN_TEST_ADDRESS();
        aborts_if !exists<UpgradeModuleCapabilities<TokenT>>(token_address);
    }
}
}