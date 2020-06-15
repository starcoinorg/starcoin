// Test user-defined token
//! account: alice
//! account: bob

//! sender: alice

module Token {

    resource struct Coin<AssetType: copyable> {
        type: AssetType,
        value: u64,
    }

    // control the minting/creation in the defining module of `ATy`
    public fun create<ATy: copyable>(type: ATy, value: u64): Coin<ATy> {
        Coin { type, value }
    }

    public fun value<ATy: copyable>(coin: &Coin<ATy>): u64 {
        coin.value
    }

    public fun split<ATy: copyable>(coin: Coin<ATy>, amount: u64): (Coin<ATy>, Coin<ATy>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    public fun withdraw<ATy: copyable>(coin: &mut Coin<ATy>, amount: u64): Coin<ATy> {
        assert(coin.value >= amount, 10);
        coin.value = coin.value - amount;
        Coin { type: *&coin.type, value: amount }
    }

    public fun join<ATy: copyable>(coin1: Coin<ATy>, coin2: Coin<ATy>): Coin<ATy> {
        deposit(&mut coin1, coin2);
        coin1
    }

    public fun deposit<ATy: copyable>(coin: &mut Coin<ATy>, check: Coin<ATy>) {
        let Coin { value, type } = check;
        assert(&coin.type == &type, 42);
        coin.value = coin.value + value;
    }

    public fun destroy_zero<ATy: copyable>(coin: Coin<ATy>) {
        let Coin { value, type: _ } = coin;
        assert(value == 0, 11)
    }

}

//! new-transaction
//! sender: bob

module ToddNickles {
    use {{alice}}::Token;
    use 0x1::Signer;

    struct T {}

    resource struct Wallet {
        nickles: Token::Coin<T>,
    }

    public fun init(account: &signer) {
        assert(Signer::address_of(account) == {{bob}}, 42);
        move_to(account, Wallet { nickles: Token::create(T{}, 0) })
    }

    public fun mint(account: &signer): Token::Coin<T> {
        assert(Signer::address_of(account) == {{bob}}, 42);
        Token::create(T{}, 5)
    }

    public fun destroy(c: Token::Coin<T>) acquires Wallet {
        Token::deposit(&mut borrow_global_mut<Wallet>({{bob}}).nickles, c)
    }

}
