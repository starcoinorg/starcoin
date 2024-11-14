/// A proposal module which is used to modify Token's DAO configuration.
module starcoin_framework::dao_modify_config_proposal {
    use starcoin_framework::stc_util;
    use starcoin_framework::signer;
    use starcoin_framework::on_chain_config;
    use starcoin_framework::dao;
    use starcoin_framework::error;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    /// A wrapper of `on_chain_config::ModifyConfigCapability<dao::DaoConfig<TokenT>>`.
    struct DaoConfigModifyCapability<phantom TokenT> has key {
        cap: on_chain_config::ModifyConfigCapability<dao::DaoConfig<TokenT>>,
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;
    const ERR_QUORUM_RATE_INVALID: u64 = 402;

    /// a proposal action to update dao config.
    /// if any field is `0`, that means the proposal want to update.
    struct DaoConfigUpdate has copy, drop, store {
        /// new voting delay setting.
        voting_delay: u64,
        /// new voting period setting.
        voting_period: u64,
        /// new voting quorum rate setting.
        voting_quorum_rate: u8,
        /// new min action delay setting.
        min_action_delay: u64,
    }

    /// Plugin method of the module.
    /// Should be called by token issuer.
    public fun plugin<TokenT: copy + drop + store>(signer: &signer) {
        let token_issuer = stc_util::token_issuer<TokenT>();
        assert!(signer::address_of(signer) == token_issuer, error::unauthenticated(ERR_NOT_AUTHORIZED));
        let dao_config_modify_cap = on_chain_config::extract_modify_config_capability<
            dao::DaoConfig<TokenT>,
        >(signer);
        assert!(
            on_chain_config::account_address(&dao_config_modify_cap) == token_issuer,
            error::unauthenticated(ERR_NOT_AUTHORIZED)
        );
        let cap = DaoConfigModifyCapability { cap: dao_config_modify_cap };
        move_to(signer, cap);
    }

    spec plugin {
        use starcoin_framework::signer;
        use starcoin_framework::option;

        pragma aborts_if_is_partial = false;
        let sender = signer::address_of(signer);
        aborts_if sender != @0x2;
        include on_chain_config::AbortsIfCapNotExist<dao::DaoConfig<TokenT>> { address: sender };

        let config_cap =
            on_chain_config::spec_cap<dao::DaoConfig<TokenT>>(sender);
        aborts_if option::is_none(config_cap);
        aborts_if option::borrow(config_cap).account_address != sender;
        aborts_if exists<DaoConfigModifyCapability<TokenT>>(sender);
        ensures exists<DaoConfigModifyCapability<TokenT>>(sender);
    }

    /// Entrypoint for the proposal.
    public entry fun propose<TokenT>(
        signer: signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
        exec_delay: u64,
    ) {
        assert!(voting_quorum_rate <= 100, error::invalid_argument(ERR_QUORUM_RATE_INVALID));
        let action = DaoConfigUpdate {
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        };
        dao::propose<TokenT, DaoConfigUpdate>(&signer, action, exec_delay);
    }
    spec propose {
        use starcoin_framework::timestamp;
        use starcoin_framework::system_addresses;
        use starcoin_framework::signer;
        use starcoin_framework::dao;

        pragma aborts_if_is_partial = false;
        aborts_if voting_quorum_rate > 100;

        // copy from dao::propose spec.
        include dao::AbortIfDaoConfigNotExist<TokenT>;
        include dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if exec_delay > 0 && exec_delay < dao::spec_dao_config<TokenT>().min_action_delay;
        include dao::CheckQuorumVotes<TokenT>;
        let sender = signer::address_of(signer);
        aborts_if exists<dao::Proposal<TokenT, DaoConfigUpdate>>(sender);
    }

    /// Once the proposal is agreed, anyone can call the method to make the proposal happen.
    public entry fun execute<TokenT>(proposer_address: address, proposal_id: u64)
    acquires DaoConfigModifyCapability {
        let DaoConfigUpdate {
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        } = dao::extract_proposal_action<TokenT, DaoConfigUpdate>(proposer_address, proposal_id);
        let cap = borrow_global_mut<DaoConfigModifyCapability<TokenT>>(
            stc_util::token_issuer<TokenT>(),
        );
        dao::modify_dao_config(
            &mut cap.cap,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        );
    }
    spec execute {
        pragma aborts_if_is_partial = true;
        // let expected_states = vec(6);
        // include dao::CheckProposalStates<TokenT, DaoConfigUpdate>{expected_states};
        aborts_if !exists<DaoConfigModifyCapability<TokenT>>(@0x2);
    }
}