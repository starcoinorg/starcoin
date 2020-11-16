script {
    use 0x1::Token;
    use 0x1::Box;
    use 0x1::Account;
    use 0x1::Offer;

    fun mint_and_split_by_linear_key<Token>(
        signer: &signer,
        for_address: address,
        amount: u128,
        lock_period: u64,
    ) {
        // 1. take key: LinearTimeMintKey<Token>
        let mint_key = Box::take<Token::LinearTimeMintKey<Token>>(signer);

        // 2. mint token
        let (tokens, new_mint_key) = Token::split_linear_key<Token>(&mut mint_key, amount);

        // 3. deposit
        Account::deposit_to_self(signer, tokens);

        // 4. put or destroy key
        if (Token::is_empty_key(&mint_key)) {
            Token::destroy_empty_key(mint_key);
        } else {
            Box::put(signer, mint_key);
        };

        // 5. offer
        Offer::create<Token::LinearTimeMintKey<Token>>(signer, new_mint_key, for_address, lock_period);
    }

    spec fun mint_and_split_by_linear_key {
        pragma verify = false;
    }
}
