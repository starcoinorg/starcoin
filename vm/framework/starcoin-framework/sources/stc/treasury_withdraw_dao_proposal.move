/// TreasuryWithdrawDaoProposal is a dao proposal for withdraw Token from Treasury.
module starcoin_framework::treasury_withdraw_dao_proposal {

    use std::error;
    use std::signer;

    use starcoin_framework::coin;
    use starcoin_framework::dao;
    use starcoin_framework::stc_util;
    use starcoin_framework::system_addresses;
    use starcoin_framework::treasury;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict;
        pragma aborts_if_is_partial;
    }

    /// A wrapper of Token MintCapability.
    struct WrappedWithdrawCapability<phantom TokenT> has key {
        cap: treasury::WithdrawCapability<TokenT>,
    }

    /// WithdrawToken request.
    struct WithdrawToken has copy, drop, store {
        /// the receiver of withdraw tokens.
        receiver: address,
        /// how many tokens to mint.
        amount: u128,
        /// How long in milliseconds does it take for the token to be released
        period: u64,
    }

    const ERR_NOT_AUTHORIZED: u64 = 101;
    /// Only receiver can execute TreasuryWithdrawDaoProposal
    const ERR_NEED_RECEIVER_TO_EXECUTE: u64 = 102;
    /// The withdraw amount of propose is too many.
    const ERR_TOO_MANY_WITHDRAW_AMOUNT: u64 = 103;

    /// Plugin method of the module.
    /// Should be called by token issuer.
    public fun plugin<TokenT>(signer: &signer, cap: treasury::WithdrawCapability<TokenT>) {
        let token_issuer = stc_util::token_issuer<TokenT>();
        assert!(signer::address_of(signer) == token_issuer, error::not_found(ERR_NOT_AUTHORIZED));
        move_to(signer, WrappedWithdrawCapability<TokenT> { cap });
    }

    spec plugin {
        pragma aborts_if_is_partial = false;
        let sender = signer::address_of(signer);
        aborts_if sender != @0x2;
        aborts_if !exists<treasury::WithdrawCapability<TokenT>>(sender);
        aborts_if exists<WrappedWithdrawCapability<TokenT>>(sender);

        ensures !exists<treasury::WithdrawCapability<TokenT>>(sender);
        ensures exists<WrappedWithdrawCapability<TokenT>>(sender);
    }


    /// Entrypoint for the proposal.
    public fun propose_withdraw<TokenT>(
        signer: &signer,
        receiver: address,
        amount: u128,
        period: u64,
        exec_delay: u64
    ) {
        let quorum_votes = dao::quorum_votes<TokenT>();
        assert!(amount <= quorum_votes, error::invalid_argument(ERR_TOO_MANY_WITHDRAW_AMOUNT));
        dao::propose<TokenT, WithdrawToken>(
            signer,
            WithdrawToken { receiver, amount, period },
            exec_delay,
        );
    }

    spec propose_withdraw {
        use starcoin_framework::dao;
        use starcoin_framework::timestamp;
        use starcoin_framework::system_addresses;

        pragma aborts_if_is_partial = false;
        let quorum_votes = dao::spec_quorum_votes<TokenT>();
        aborts_if amount > quorum_votes;
        // copy from dao::propose spec.
        include dao::AbortIfDaoConfigNotExist<TokenT>;
        include dao::AbortIfDaoInfoNotExist<TokenT>;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if exec_delay > 0 && exec_delay < dao::spec_dao_config<TokenT>().min_action_delay;
        include dao::CheckQuorumVotes<TokenT>;
        let sender = signer::address_of(signer);
        aborts_if exists<dao::Proposal<TokenT, WithdrawToken>>(sender);
    }

    /// Once the proposal is agreed, anyone can call the method to make the proposal happen.
    public fun execute_withdraw_proposal<TokenT>(
        signer: &signer,
        proposer_address: address,
        proposal_id: u64,
    ) acquires WrappedWithdrawCapability {
        let WithdrawToken { receiver, amount, period } = dao::extract_proposal_action<TokenT, WithdrawToken>(
            proposer_address,
            proposal_id,
        );
        assert!(receiver == signer::address_of(signer), error::not_found(ERR_NEED_RECEIVER_TO_EXECUTE));
        let cap =
            borrow_global_mut<WrappedWithdrawCapability<TokenT>>(stc_util::token_issuer<TokenT>());
        let linear_cap =
            treasury::issue_linear_withdraw_capability<TokenT>(&mut cap.cap, amount, period);
        treasury::add_linear_withdraw_capability(signer, linear_cap);
    }

    spec execute_withdraw_proposal {
        use starcoin_framework::option;

        pragma aborts_if_is_partial = true;
        let expected_states = vec<u8>(6);
        include dao::CheckProposalStates<TokenT, WithdrawToken> { expected_states };
        let proposal = global<dao::Proposal<TokenT, WithdrawToken>>(proposer_address);
        aborts_if option::is_none(proposal.action);
        aborts_if !exists<WrappedWithdrawCapability<TokenT>>(@0x2);
    }

    /// Provider a port for get block reward STC from Treasury, only genesis account can invoke this function.
    /// The TreasuryWithdrawCapability is locked in TreasuryWithdrawDaoProposal, and only can withdraw by DAO proposal.
    /// This approach is not graceful, but restricts the operation to genesis accounts only, so there are no security issues either.
    public fun withdraw_for_block_reward<TokenT>(
        signer: &signer,
        reward: u128
    ): coin::Coin<TokenT> acquires WrappedWithdrawCapability {
        system_addresses::assert_starcoin_framework(signer);
        let cap = borrow_global_mut<WrappedWithdrawCapability<TokenT>>(signer::address_of(signer));
        treasury::withdraw_with_capability(&mut cap.cap, reward)
    }
}