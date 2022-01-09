address StarcoinFramework {
/// A proposal module which is used to modify Token's DAO configuration.
module ModifyDaoConfigProposal {
    // use StarcoinFramework::Config;
    use StarcoinFramework::Token;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Config;
    use StarcoinFramework::Dao;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Option;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    /// A wrapper of `Config::ModifyConfigCapability<Dao::DaoConfig<TokenT>>`.
    struct DaoConfigModifyCapability<phantom TokenT: copy + drop + store> has key {
        cap: Config::ModifyConfigCapability<Dao::DaoConfig<TokenT>>,
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
        let token_issuer = Token::token_address<TokenT>();
        assert!(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let dao_config_modify_cap = Config::extract_modify_config_capability<
            Dao::DaoConfig<TokenT>,
        >(signer);
        assert!(Config::account_address(&dao_config_modify_cap) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let cap = DaoConfigModifyCapability { cap: dao_config_modify_cap };
        move_to(signer, cap);
    }

    spec plugin {
        pragma aborts_if_is_partial = false;
        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        include Config::AbortsIfCapNotExist<Dao::DaoConfig<TokenT>>{account: sender};
        let config_cap = Config::spec_cap<Dao::DaoConfig<TokenT>>(sender);
        aborts_if Option::is_none(config_cap);
        aborts_if Option::borrow(config_cap).account_address != sender;
        aborts_if exists<DaoConfigModifyCapability<TokenT>>(sender);
        ensures exists<DaoConfigModifyCapability<TokenT>>(sender);
    }

    /// Entrypoint for the proposal.
    public(script) fun propose<TokenT: copy + drop + store>(
        signer: signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
        exec_delay: u64,
    ) {
        assert!(voting_quorum_rate <= 100, Errors::invalid_argument(ERR_QUORUM_RATE_INVALID));
        let action = DaoConfigUpdate {
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        };
        Dao::propose<TokenT, DaoConfigUpdate>(&signer, action, exec_delay);
    }
    spec propose {
        use StarcoinFramework::Timestamp;
        use StarcoinFramework::CoreAddresses;
        pragma aborts_if_is_partial = false;
        aborts_if voting_quorum_rate > 100;

        // copy from Dao::propose spec.
        include Dao::AbortIfDaoConfigNotExist<TokenT>;
        include Dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if exec_delay > 0 && exec_delay < Dao::spec_dao_config<TokenT>().min_action_delay;
        include Dao::CheckQuorumVotes<TokenT>;
        let sender = Signer::address_of(signer);
        aborts_if exists<Dao::Proposal<TokenT, DaoConfigUpdate>>(sender);

    }
    /// Once the proposal is agreed, anyone can call the method to make the proposal happen.
    public(script) fun execute<TokenT: copy + drop + store>(proposer_address: address, proposal_id: u64)
    acquires DaoConfigModifyCapability {
        let DaoConfigUpdate {
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        } = Dao::extract_proposal_action<TokenT, DaoConfigUpdate>(proposer_address, proposal_id);
        let cap = borrow_global_mut<DaoConfigModifyCapability<TokenT>>(
            Token::token_address<TokenT>(),
        );
        Dao::modify_dao_config(
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
        // include Dao::CheckProposalStates<TokenT, DaoConfigUpdate>{expected_states};
        aborts_if !exists<DaoConfigModifyCapability<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }
}
}