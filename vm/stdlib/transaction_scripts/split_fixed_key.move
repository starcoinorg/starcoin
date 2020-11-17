script {
    use 0x1::Token;
    use 0x1::Box;
    use 0x1::Offer;

    fun split_fixed_key<Token>(
        signer: &signer,
        for_address: address,
        amount: u128,
        lock_period: u64,
    ) {
        // 1. take key: FixedTimeMintKey<Token>
        let mint_key = Box::take<Token::FixedTimeMintKey<Token>>(signer);

        // 2.
        let new_mint_key = Token::split_fixed_key<Token>(&mut mint_key, amount);

        // 3. put key
        Box::put(signer, mint_key);

        // 4. offer
        Offer::create<Token::FixedTimeMintKey<Token>>(signer, new_mint_key, for_address, lock_period);
    }

    spec fun split_fixed_key {
        pragma verify = false;
    }
}
