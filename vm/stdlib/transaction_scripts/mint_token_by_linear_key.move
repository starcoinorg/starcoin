script {
    use 0x1::Token;
    use 0x1::Box;
    use 0x1::Account;

    fun mint_token_by_linear_key<Token>(
        signer: &signer,
    ) {
        // 1. take key: LinearTimeMintKey<Token>
        let mint_key = Box::take<Token::LinearTimeMintKey<Token>>(signer);

        // 2. mint token
        let tokens = Token::mint_with_linear_key<Token>(&mut mint_key);

        // 3. deposit
        Account::deposit_to_self(signer, tokens);

        // 4. put or destroy key
        if (Token::is_empty_key(&mint_key)) {
            Token::destroy_empty_key(mint_key);
        } else {
            Box::put(signer, mint_key);
        }
    }

    spec fun mint_token_by_linear_key {
        pragma verify = false;
    }
}
