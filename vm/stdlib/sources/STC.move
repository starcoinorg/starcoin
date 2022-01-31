address StarcoinFramework {
/// STC is the token of Starcoin blockchain.
/// It uses apis defined in the `Token` module.
module STC {
    use StarcoinFramework::Token::{Self, Token};
    use StarcoinFramework::Dao;
    use StarcoinFramework::ModifyDaoConfigProposal;
    use StarcoinFramework::UpgradeModuleDaoProposal;
    use StarcoinFramework::PackageTxnManager;
    use StarcoinFramework::OnChainConfigDao;
    use StarcoinFramework::TransactionPublishOption;
    use StarcoinFramework::VMConfig;
    use StarcoinFramework::ConsensusConfig;
    use StarcoinFramework::RewardConfig;
    use StarcoinFramework::TransactionTimeoutConfig;
    use StarcoinFramework::Treasury;
    use StarcoinFramework::CoreAddresses;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    /// STC token marker.
    struct STC has copy, drop, store { }

    /// precision of STC token.
    const PRECISION: u8 = 9;

    /// Burn capability of STC.
    struct SharedBurnCapability has key, store {
        cap: Token::BurnCapability<STC>,
    }

    /// STC initialization.
    public fun initialize(
        account: &signer,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ) {
        Token::register_token<STC>(account, PRECISION);
        let burn_cap = Token::remove_burn_capability<STC>(account);
        move_to(account, SharedBurnCapability { cap: burn_cap });
        Dao::plugin<STC>(
            account,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        );
        ModifyDaoConfigProposal::plugin<STC>(account);
        let upgrade_plan_cap = PackageTxnManager::extract_submit_upgrade_plan_cap(account);
        UpgradeModuleDaoProposal::plugin<STC>(
            account,
            upgrade_plan_cap,
        );
        // the following configurations are gov-ed by Dao.
        OnChainConfigDao::plugin<STC, TransactionPublishOption::TransactionPublishOption>(account);
        OnChainConfigDao::plugin<STC, VMConfig::VMConfig>(account);
        OnChainConfigDao::plugin<STC, ConsensusConfig::ConsensusConfig>(account);
        OnChainConfigDao::plugin<STC, RewardConfig::RewardConfig>(account);
        OnChainConfigDao::plugin<STC, TransactionTimeoutConfig::TransactionTimeoutConfig>(account);
    }

    spec initialize {
        include Token::RegisterTokenAbortsIf<STC>{precision: PRECISION};
    }

    public fun upgrade_from_v1_to_v2(account: &signer,total_amount: u128,): Treasury::WithdrawCapability<STC> {
        CoreAddresses::assert_genesis_address(account);

        // Mint all stc, and destroy mint capability
        let total_stc = Token::mint<STC>(account, total_amount-Token::market_cap<STC>());
        let withdraw_cap = Treasury::initialize(account, total_stc);
        let mint_cap = Token::remove_mint_capability<STC>(account);
        Token::destroy_mint_capability(mint_cap);
        withdraw_cap
    }

    spec upgrade_from_v1_to_v2 {
        pragma verify = false;
    }

    /// STC initialization.
    public fun initialize_v2(
        account: &signer,
        total_amount: u128,
        voting_delay: u64,
        voting_period: u64,
        voting_quorum_rate: u8,
        min_action_delay: u64,
    ): Treasury::WithdrawCapability<STC> {
        Token::register_token<STC>(account, PRECISION);

        // Mint all stc, and destroy mint capability

        let total_stc = Token::mint<STC>(account, total_amount);
        let withdraw_cap = Treasury::initialize(account, total_stc);
        let mint_cap = Token::remove_mint_capability<STC>(account);
        Token::destroy_mint_capability(mint_cap);

        let burn_cap = Token::remove_burn_capability<STC>(account);
        move_to(account, SharedBurnCapability { cap: burn_cap });
        Dao::plugin<STC>(
            account,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay,
        );
        ModifyDaoConfigProposal::plugin<STC>(account);
        let upgrade_plan_cap = PackageTxnManager::extract_submit_upgrade_plan_cap(account);
        UpgradeModuleDaoProposal::plugin<STC>(
            account,
            upgrade_plan_cap,
        );
        // the following configurations are gov-ed by Dao.
        OnChainConfigDao::plugin<STC, TransactionPublishOption::TransactionPublishOption>(account);
        OnChainConfigDao::plugin<STC, VMConfig::VMConfig>(account);
        OnChainConfigDao::plugin<STC, ConsensusConfig::ConsensusConfig>(account);
        OnChainConfigDao::plugin<STC, RewardConfig::RewardConfig>(account);
        OnChainConfigDao::plugin<STC, TransactionTimeoutConfig::TransactionTimeoutConfig>(account);
        withdraw_cap
    }

    spec initialize_v2 {
        include Token::RegisterTokenAbortsIf<STC>{precision: PRECISION};
    }

    /// Returns true if `TokenType` is `STC::STC`
    public fun is_stc<TokenType: store>(): bool {
        Token::is_same_token<STC, TokenType>()
    }

    spec is_stc {
    }

    /// Burn STC tokens.
    /// It can be called by anyone.
    public fun burn(token: Token<STC>) acquires SharedBurnCapability {
        let cap = borrow_global<SharedBurnCapability>(token_address());
        Token::burn_with_capability(&cap.cap, token);
    }

    spec burn {
        aborts_if Token::spec_abstract_total_value<STC>() - token.value < 0;
        aborts_if !exists<SharedBurnCapability>(Token::SPEC_TOKEN_TEST_ADDRESS());
    }

    /// Return STC token address.
    public fun token_address(): address {
        Token::token_address<STC>()
    }

    spec token_address {
    }
}
}