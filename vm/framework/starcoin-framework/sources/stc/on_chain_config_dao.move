/// OnChainConfigDao is a DAO proposal for modify onchain configuration.
module starcoin_framework::on_chain_config_dao {
    use std::error;
    use std::signer;

    use starcoin_framework::dao;
    use starcoin_framework::stc_util;
    use starcoin_framework::on_chain_config;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    /// A wrapper of `Config::ModifyConfigCapability<ConfigT>`.
    struct WrappedConfigModifyCapability<phantom TokenT, ConfigT: copy + drop + store> has key {
        cap: on_chain_config::ModifyConfigCapability<ConfigT>,
    }

    /// request of updating configuration.
    struct OnChainConfigUpdate<ConfigT: copy + drop + store> has copy, drop, store {
        value: ConfigT,
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;

    /// Plugin method of the module.
    /// Should be called by token issuer.
    public fun plugin<TokenT, ConfigT: copy + drop + store>(signer: &signer) {
        let token_issuer = stc_util::token_issuer<TokenT>(); // coin::token_address<TokenT>();
        assert!(signer::address_of(signer) == token_issuer, error::not_found(ERR_NOT_AUTHORIZED));
        let config_modify_cap = on_chain_config::extract_modify_config_capability<ConfigT>(signer);
        let cap = WrappedConfigModifyCapability<TokenT, ConfigT> { cap: config_modify_cap };
        move_to(signer, cap);
    }

    spec plugin {
        use starcoin_framework::signer;

        pragma aborts_if_is_partial = false;
        let sender = signer::address_of(signer);
        aborts_if sender != @0x2;
        include on_chain_config::AbortsIfCapNotExist<ConfigT> { address: sender };
        aborts_if exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(sender);
        ensures exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(sender);
    }

    /// issue a proposal to update config of ConfigT goved by TokenT
    public fun propose_update<TokenT, ConfigT: copy + drop + store>(
        signer: &signer,
        new_config: ConfigT,
        exec_delay: u64,
    ) {
        dao::propose<TokenT, OnChainConfigUpdate<ConfigT>>(
            signer,
            OnChainConfigUpdate { value: new_config },
            exec_delay,
        );
    }

    spec propose_update {
        use starcoin_framework::timestamp;
        use starcoin_framework::system_addresses;
        use starcoin_framework::signer;

        pragma aborts_if_is_partial = false;

        // copy from Dao::propose spec.
        include dao::AbortIfDaoConfigNotExist<TokenT>;
        include dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if exec_delay > 0 && exec_delay < dao::spec_dao_config<TokenT>().min_action_delay;
        include dao::CheckQuorumVotes<TokenT>;
        let sender = signer::address_of(signer);
        aborts_if exists<dao::Proposal<TokenT, OnChainConfigUpdate<ConfigT>>>(sender);
    }

    /// Once the proposal is agreed, anyone can call the method to make the proposal happen.
    /// Caller need to make sure that the proposal of `proposal_id` under `proposal_address` is
    /// the kind of this proposal module.
    public fun execute<TokenT, ConfigT: copy + drop + store>(
        proposer_address: address,
        proposal_id: u64,
    ) acquires WrappedConfigModifyCapability {
        let OnChainConfigUpdate { value } = dao::extract_proposal_action<
            TokenT,
            OnChainConfigUpdate<ConfigT>,
        >(proposer_address, proposal_id);
        let cap = borrow_global_mut<WrappedConfigModifyCapability<TokenT, ConfigT>>(
            stc_util::token_issuer<TokenT>(),
        );
        on_chain_config::set_with_capability(&mut cap.cap, value);
    }

    spec execute {
        pragma aborts_if_is_partial = true;
        let expected_states = vec<u8>(6);
        include dao::CheckProposalStates<TokenT, OnChainConfigUpdate<ConfigT>> { expected_states };
        aborts_if !exists<WrappedConfigModifyCapability<TokenT, ConfigT>>(@0x2);
    }
}