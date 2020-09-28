address 0x1 {
module STC {
    use 0x1::Token::{Self, Token};
    use 0x1::Dao;
    use 0x1::ModifyDaoConfigProposal;
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::PackageTxnManager;
    use 0x1::OnChainConfigDao;
    use 0x1::TransactionPublishOption;
    use 0x1::VMConfig;
    use 0x1::ConsensusConfig;
    use 0x1::Version;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    struct STC { }

    // TODO: make decision of how long the factor should be
    /// scaling_factor = 10^9
    const BASE_SCALING_FACTOR: u128 = 1000000000;
    /// fractional_part = 10^3
    const FRACTIONAL_PART: u128 = 1000;

    resource struct SharedBurnCapability {
        cap: Token::BurnCapability<STC>,
    }

    public fun initialize(account: &signer) {
        Token::register_token<STC>(account, BASE_SCALING_FACTOR, FRACTIONAL_PART);
        let burn_cap = Token::remove_burn_capability<STC>(account);
        move_to(account, SharedBurnCapability { cap: burn_cap });
        Dao::plugin<STC>(
            account,
            Dao::default_voting_delay(),
            Dao::default_voting_period(),
            Dao::default_voting_quorum_rate(),
            Dao::default_min_action_delay(),
        );
        ModifyDaoConfigProposal::plugin<STC>(account);
        UpgradeModuleDaoProposal::plugin<STC>(account);
        let upgrade_plan_cap = PackageTxnManager::extract_submit_upgrade_plan_cap(account);
        UpgradeModuleDaoProposal::delegate_module_upgrade_capability<STC>(
            account,
            upgrade_plan_cap,
        );
        // the following configurations are gov-ed by Dao.
        OnChainConfigDao::plugin<STC, TransactionPublishOption::TransactionPublishOption>(account);
        OnChainConfigDao::plugin<STC, VMConfig::VMConfig>(account);
        OnChainConfigDao::plugin<STC, Version::Version>(account);
        OnChainConfigDao::plugin<STC, ConsensusConfig::ConsensusConfig>(account);
    }

    spec fun initialize {
        // Todo: fix name_of()
        pragma verify = false;
    }

    /// Returns true if `TokenType` is `STC::STC`
    public fun is_stc<TokenType>(): bool {
        Token::is_same_token<STC, TokenType>()
    }

    spec fun is_stc {
    }

    public fun burn(token: Token<STC>) acquires SharedBurnCapability {
        let cap = borrow_global<SharedBurnCapability>(token_address());
        Token::burn_with_capability(&cap.cap, token);
    }

    spec fun burn {
        // Todo: fix name_of()
        pragma verify = false;
    }

    public fun token_address(): address {
        Token::token_address<STC>()
    }

    spec fun token_address {
    }
}
}