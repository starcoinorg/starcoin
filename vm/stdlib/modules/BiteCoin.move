address 0x1 {
module BiteCoin {
    use 0x1::Token;
    use 0x1::Block;
    use 0x1::Signer;
    use 0x1::Balance;
    use 0x1::TrivalTransfer;
    use 0x1::FreelyBurn;

    /// CoinType
    resource struct T { }

    resource struct MintManager {
        mint_cap: Token::MintCapability<T>,
        mint_times: u64,
        start_block_height: u64,
    }

    // const TOKEN_ADDRESS: address = 0x1;
    // const COIN: u64 = 100000000;
    // /// Total Supply: 21 million * 2.
    // const TOTAL_SUPPLY: u64 = 21000000 * 2;
    // /// Initial Mint: 21 million. (50% of the total supply)
    // const INITIAL_MINT_AMOUNT: u64 = 21000000;
    // /// every 21w block, can mint 21w * `block_subsidy` to issuer.
    // const HALVING_INTERVAL: u64 = 210000;
    // const INITIAL_SUBSIDY: u64 = 50;
    fun token_address(): address {
        0x1
    }

    fun total_supply(): u64 {
        21000000 * 2
    }

    fun initial_mint_amount(): u64 {
        21000000
    }

    fun halving_interval(): u64 {
        210000
    }

    fun initial_subsidy(): u64 {
        50
    }

    public fun initialize(signer: &signer) {
        assert(Signer::address_of(signer) == token_address(), 401);
        let t = T {};
        // register currency.
        Token::register_currency<T>(signer, &t, 1000, 1000);
        Balance::accept_token<T>(signer);
        TrivalTransfer::plug_in<T>(signer, &t);
        // Mint to myself at the beginning.
        let minted_token = Token::mint<T>(
            signer,
            initial_mint_amount() * 100000000,
            token_address(),
        );
        Balance::deposit_to(token_address(), minted_token);
        let mint_cap = Token::remove_my_mint_capability<T>(signer);
        let block_height = Block::get_current_block_height();
        let mint_manager = MintManager {
            mint_cap,
            mint_times: 0,
            start_block_height: block_height,
        };
        move_to(signer, mint_manager);
        // plug in FreelyBurn.
        FreelyBurn::plug_in<T>(signer, &t);
        // destroy T, so that no one can mint. (except this contract)
        let T{  } = t;
    }

    /// Anyone can trigger a mint action if it's time to mint.
    public fun trigger_mint(_signer: &signer) acquires MintManager {
        let current_block_height = Block::get_current_block_height();
        let mint_manager = borrow_global_mut<MintManager>(token_address());
        let interval = current_block_height - mint_manager.start_block_height;
        let halvings = interval / halving_interval();
        let mint_times = mint_manager.mint_times;
        assert(mint_times <= halvings, 500);
        if (halvings == mint_times) {
            return
        };
        if (mint_times >= 64) {
            return
        };
        let block_subsidy = initial_subsidy() * 100000000 >> (mint_times as u8);
        let mint_amount = halving_interval() * block_subsidy;
        mint_manager.mint_times = mint_manager.mint_times + 1;
        if (mint_amount == 0) {
            return
        };
        let minted_token = Token::mint_with_capability<T>(
            mint_amount,
            token_address(),
            &mint_manager.mint_cap,
        );
        Balance::deposit_to(token_address(), minted_token);
    }

    /// Get the balance of `user`
    public fun balance(user: address): u64 {
        Balance::balance<T>(user)
    }

    public fun transfer_to(signer: &signer, receiver: address, amount: u64) {
        TrivalTransfer::transfer<T>(signer, token_address(), receiver, amount);
    }

    /// Anyone can burn his money.
    public fun burn(signer: &signer, amount: u64) {
        let coins = TrivalTransfer::withdraw<T>(signer, token_address(), amount);
        FreelyBurn::burn<T>(token_address(), coins);
    }
}
}