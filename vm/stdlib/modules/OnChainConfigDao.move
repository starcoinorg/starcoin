address 0x1 {
module OnChainConfigDao {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Config;
    use 0x1::Dao;
    use 0x1::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    // use 0x1::CoreAddresses;
    resource struct WrappedConfigModifyCapability<TokenT, ConfigT: copyable> {
        cap: Config::ModifyConfigCapability<ConfigT>,
    }

    struct OnChainConfigUpdate<ConfigT: copyable> {
        value: ConfigT,
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;

    public fun plugin<TokenT, ConfigT: copyable>(signer: &signer) {
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let config_modify_cap = Config::extract_modify_config_capability<ConfigT>(signer);
        let cap = WrappedConfigModifyCapability<TokenT, ConfigT> { cap: config_modify_cap };
        move_to(signer, cap);
    }
    spec fun plugin {
        pragma aborts_if_is_partial = false;
        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        include Config::AbortsIfCapNotExist<ConfigT>{account: sender};
        aborts_if exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(sender);
        ensures exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(sender);
    }

    /// issue a proposal to update config of ConfigT goved by TokenT
    public fun propose_update<TokenT: copyable, ConfigT: copyable>(
        signer: &signer,
        new_config: ConfigT,
        exec_delay: u64,
    ) {
        Dao::propose<TokenT, OnChainConfigUpdate<ConfigT>>(
            signer,
            OnChainConfigUpdate { value: new_config },
            exec_delay,
        );
    }

    spec fun propose_update {
        use 0x1::Timestamp;
        use 0x1::CoreAddresses;
        pragma aborts_if_is_partial = false;

        // copy from Dao::propose spec.
        include Dao::AbortIfDaoConfigNotExist<TokenT>;
        include Dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if exec_delay > 0 && exec_delay < Dao::spec_dao_config<TokenT>().min_action_delay;
        include Dao::CheckQuorumVotes<TokenT>;
        let sender = Signer::spec_address_of(signer);
        aborts_if exists<Dao::Proposal<TokenT, OnChainConfigUpdate<ConfigT>>>(sender);
    }

    public fun execute<TokenT: copyable, ConfigT: copyable>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires WrappedConfigModifyCapability {
        let OnChainConfigUpdate { value } = Dao::extract_proposal_action<
            TokenT,
            OnChainConfigUpdate<ConfigT>,
        >(proposer_address, proposal_id);
        let cap = borrow_global_mut<WrappedConfigModifyCapability<TokenT, ConfigT>>(
            Token::token_address<TokenT>(),
        );
        Config::set_with_capability(&mut cap.cap, value);
    }
    spec fun execute {
        pragma aborts_if_is_partial = true;
        let expected_states = singleton_vector(6);
        include Dao::CheckProposalStates<TokenT, OnChainConfigUpdate<ConfigT>>{expected_states};
        aborts_if !exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }
}
}