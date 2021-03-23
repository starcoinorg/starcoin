address 0x1 {
module MintScripts {
    use 0x1::Token;
    use 0x1::Collection;
    use 0x1::Account;
    use 0x1::Offer;

    public(script) fun mint_and_split_by_linear_key<Token: store>(
        signer: &signer,
        for_address: address,
        amount: u128,
        lock_period: u64,
    ) {
        // 1. take key: LinearTimeMintKey<Token>
        let mint_key = Collection::take<Token::LinearTimeMintKey<Token>>(signer);

        // 2. mint token
        let (tokens, new_mint_key) = Token::split_linear_key<Token>(&mut mint_key, amount);

        // 3. deposit
        Account::deposit_to_self(signer, tokens);

        // 4. put or destroy key
        if (Token::is_empty_key(&mint_key)) {
            Token::destroy_empty_key(mint_key);
        } else {
            Collection::put(signer, mint_key);
        };

        // 5. offer
        Offer::create<Token::LinearTimeMintKey<Token>>(signer, new_mint_key, for_address, lock_period);
    }

    spec fun mint_and_split_by_linear_key {
        pragma verify = false;
    }

    public(script) fun mint_token_by_fixed_key<Token: store>(
        signer: &signer,
    ) {
        // 1. take key: FixedTimeMintKey<Token>
        let mint_key = Collection::take<Token::FixedTimeMintKey<Token>>(signer);

        // 2. mint token
        let tokens = Token::mint_with_fixed_key<Token>(mint_key);

        // 3. deposit
        Account::deposit_to_self(signer, tokens);
    }

    spec fun mint_token_by_fixed_key {
        pragma verify = false;
    }

    public(script) fun mint_token_by_linear_key<Token: store>(
        signer: &signer,
    ) {
        // 1. take key: LinearTimeMintKey<Token>
        let mint_key = Collection::take<Token::LinearTimeMintKey<Token>>(signer);

        // 2. mint token
        let tokens = Token::mint_with_linear_key<Token>(&mut mint_key);

        // 3. deposit
        Account::deposit_to_self(signer, tokens);

        // 4. put or destroy key
        if (Token::is_empty_key(&mint_key)) {
            Token::destroy_empty_key(mint_key);
        } else {
            Collection::put(signer, mint_key);
        }
    }

    spec fun mint_token_by_linear_key {
        pragma verify = false;
    }

    public(script) fun split_fixed_key<Token: store>(
        signer: &signer,
        for_address: address,
        amount: u128,
        lock_period: u64,
    ) {
        // 1. take key: FixedTimeMintKey<Token>
        let mint_key = Collection::take<Token::FixedTimeMintKey<Token>>(signer);

        // 2.
        let new_mint_key = Token::split_fixed_key<Token>(&mut mint_key, amount);

        // 3. put key
        Collection::put(signer, mint_key);

        // 4. offer
        Offer::create<Token::FixedTimeMintKey<Token>>(signer, new_mint_key, for_address, lock_period);
    }

    spec fun split_fixed_key {
        pragma verify = false;
    }
}
}