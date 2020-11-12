address 0x1 {
module ModifyDaoConfigProposal {
    // use 0x1::Config;
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

    resource struct DaoConfigModifyCapability<TokenT: copyable> {
        cap: Config::ModifyConfigCapability<Dao::DaoConfig<TokenT>>,
    }

    const ERR_NOT_AUTHORIZED: u64 = 401;
    const ERR_QUROM_RATE_INVALID: u64 = 402;

    /// a proposal action to udpate dao config.
    /// if any field is `0`, that means the proposal want to update.
    struct DaoConfigUpdate {
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    }

    public fun plugin<TokenT: copyable>(signer: &signer) {
        let token_issuer = Token::token_address<TokenT>();
        assert(Signer::address_of(signer) == token_issuer, Errors::requires_address(ERR_NOT_AUTHORIZED));
        let dao_config_moidify_cap = Config::extract_modify_config_capability<
            Dao::DaoConfig<TokenT>,
        >(signer);
        // TODO: assert cap.account_address == token_issuer
        let cap = DaoConfigModifyCapability { cap: dao_config_moidify_cap };
        move_to(signer, cap);
    }

    spec fun plugin {
        pragma aborts_if_is_partial = false;
        let sender = Signer::address_of(signer);
        aborts_if sender != Token::SPEC_TOKEN_TEST_ADDRESS();
        include Config::AbortsIfCapNotExist<Dao::DaoConfig<TokenT>>{account: sender};
        aborts_if exists<DaoConfigModifyCapability<TokenT>>(sender);
        ensures exists<DaoConfigModifyCapability<TokenT>>(sender);
    }

    public fun propose<TokenT: copyable>(
        signer: &signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
        exec_delay: u64,
    ) {
        assert(voting_quorum_rate <= 100, Errors::invalid_argument(ERR_QUROM_RATE_INVALID));
        let action = DaoConfigUpdate {
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        };
        Dao::propose<TokenT, DaoConfigUpdate>(signer, action, exec_delay);
    }
    spec fun propose {
        use 0x1::Timestamp;
        use 0x1::CoreAddresses;
        pragma aborts_if_is_partial = false;
        aborts_if voting_quorum_rate > 100;

        // copy from Dao::propose spec.
        include Dao::AbortIfDaoConfigNotExist<TokenT>;
        include Dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if exec_delay > 0 && exec_delay < Dao::spec_dao_config<TokenT>().min_action_delay;
        include Dao::CheckQuorumVotes<TokenT>;
        let sender = Signer::spec_address_of(signer);
        aborts_if exists<Dao::Proposal<TokenT, DaoConfigUpdate>>(sender);

    }

    public fun execute<TokenT: copyable>(proposer_address: address, proposal_id: u64)
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
    spec fun execute {
        pragma aborts_if_is_partial = true;
        // let expected_states = singleton_vector(6);
        // include Dao::CheckProposalStates<TokenT, DaoConfigUpdate>{expected_states};
        aborts_if !exists<DaoConfigModifyCapability<TokenT>>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }
}
}