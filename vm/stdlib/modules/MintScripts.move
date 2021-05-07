address 0x1 {
module MintScripts {
    use 0x1::Errors;

    const EDEPRECATED_FUNCTION: u64 = 11;

    public(script) fun mint_and_split_by_linear_key<Token: store>(
        _signer: signer,
        _for_address: address,
        _amount: u128,
        _lock_period: u64,
    ) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec fun mint_and_split_by_linear_key {
        pragma verify = false;
    }

    public(script) fun mint_token_by_fixed_key<Token: store>(
        _signer: signer,
    ) {
       abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec fun mint_token_by_fixed_key {
        pragma verify = false;
    }

    public(script) fun mint_token_by_linear_key<Token: store>(
        _signer: signer,
    ) {
       abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec fun mint_token_by_linear_key {
        pragma verify = false;
    }

    public(script) fun split_fixed_key<Token: store>(
        _signer: signer,
        _for_address: address,
        _amount: u128,
        _lock_period: u64,
    ) {
       abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec fun split_fixed_key {
        pragma verify = false;
    }
}
}