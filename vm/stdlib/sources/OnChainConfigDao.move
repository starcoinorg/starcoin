address StarcoinFramework {
/// OnChainConfigDao is a DAO proposal for modify onchain configuration.
module OnChainConfigDao {
    use StarcoinFramework::Token;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Config;
    use StarcoinFramework::Dao;
    use StarcoinFramework::Errors;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    /// A wrapper of `Config::ModifyConfigCapability<ConfigT>`.
    struct WrappedConfigModifyCapability<phantom TokenT, ConfigT: copy + drop + store> has key {
        cap: Config::ModifyConfigCapability<ConfigT>,
    }

    /// request of updating configuration.
    struct OnChainConfigUpdate<ConfigT: copy + drop + store> has copy, drop, store {
        value: ConfigT,
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;

    /// Plugin method of the module.
    /// Should be called by token issuer.
    public fun plugin<TokenT: copy + drop + store, ConfigT: copy + drop + store>(signer: &signer) {
        let token_issuer = Token::token_address<TokenT>();
        assert!(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let config_modify_cap = Config::extract_modify_config_capability<ConfigT>(signer);
        let cap = WrappedConfigModifyCapability<TokenT, ConfigT> { cap: config_modify_cap };
        move_to(signer, cap);
    }
    spec plugin {
        pragma aborts_if_is_partial = false;
        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        include Config::AbortsIfCapNotExist<ConfigT>{account: sender};
        aborts_if exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(sender);
        ensures exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(sender);
    }

    /// issue a proposal to update config of ConfigT goved by TokenT
    public fun propose_update<TokenT: copy + drop + store, ConfigT: copy + drop + store>(
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

    spec propose_update {
        use StarcoinFramework::Timestamp;
        use StarcoinFramework::CoreAddresses;
        pragma aborts_if_is_partial = false;

        // copy from Dao::propose spec.
        include Dao::AbortIfDaoConfigNotExist<TokenT>;
        include Dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if exec_delay > 0 && exec_delay < Dao::spec_dao_config<TokenT>().min_action_delay;
        include Dao::CheckQuorumVotes<TokenT>;
        let sender = Signer::address_of(signer);
        aborts_if exists<Dao::Proposal<TokenT, OnChainConfigUpdate<ConfigT>>>(sender);
    }

    /// Once the proposal is agreed, anyone can call the method to make the proposal happen.
    /// Caller need to make sure that the proposal of `proposal_id` under `proposal_address` is
    /// the kind of this proposal module.
    public fun execute<TokenT: copy + drop + store, ConfigT: copy + drop + store>(
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
    spec execute {
        pragma aborts_if_is_partial = true;
        let expected_states = vec<u8>(6);
        include Dao::CheckProposalStates<TokenT, OnChainConfigUpdate<ConfigT>>{expected_states};
        aborts_if !exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }
}
}