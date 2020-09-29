address 0x1 {
module OnChainConfigDao {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Config;
    use 0x1::Dao;

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
        assert(Signer::address_of(signer) == token_issuer, ERR_NOT_AUTHORIZED);
        let config_moidify_cap = Config::extract_modify_config_capability<ConfigT>(signer);
        let cap = WrappedConfigModifyCapability<TokenT, ConfigT> { cap: config_moidify_cap };
        move_to(signer, cap);
    }

    /// issue a proposal to update config of ConfigT goved by TokenT
    public fun propose_update<TokenT: copyable, ConfigT: copyable>(
        signer: &signer,
        new_config: ConfigT,
    ) {
        Dao::propose<TokenT, OnChainConfigUpdate<ConfigT>>(
            signer,
            OnChainConfigUpdate { value: new_config },
            Dao::min_action_delay<TokenT>(),
        );
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
}
}