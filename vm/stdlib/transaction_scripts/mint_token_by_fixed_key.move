script {
    use 0x1::Token;
    use 0x1::Box;
    use 0x1::Account;

    fun mint_token_by_fixed_key<Token>(
        signer: &signer,
    ) {
        // 1. take key: FixedTimeMintKey<Token>
        let mint_key = Box::take<Token::FixedTimeMintKey<Token>>(signer);

        // 2. mint token
        let tokens = Token::mint_with_fixed_key<Token>(mint_key);

        // 3. deposit
        Account::deposit_to_self(signer, tokens);
    }

    spec fun mint_token_by_fixed_key {
        pragma verify = false;
    }
}
