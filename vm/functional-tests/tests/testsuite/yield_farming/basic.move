//! account: alice, 100000000000000000 0x1::STC::STC
//! account: joe
//! account: admin, 0x81144d60492982a45ba93fba47cae988, 10000000000000 0x1::STC::STC
//! account: liquidier, 10000000000000 0x1::STC::STC
//! account: exchanger

//! sender: alice
address alice = {{alice}};
module alice::TokenMock {
    use 0x1::Token;
    use 0x1::YieldFarming;
    use 0x1::Signer;

    struct Usdx has copy, drop, store {}

    struct GovModfiyParamCapability<PoolType, AssetT> has key, store {
        cap: YieldFarming::ParameterModifyCapability<PoolType, AssetT>,
    }

    struct PoolType_A has copy, drop, store {}

    struct AssetType_A has copy, drop, store { 
        value: u128 
    }

    public fun initialize(account: &signer, treasury: Token::Token<Usdx>) {
        YieldFarming::initialize<PoolType_A, Usdx>(account, treasury);
        let asset_cap = YieldFarming::initialize_asset<PoolType_A, AssetType_A>(account, 1000000000, 0);
        move_to(account, GovModfiyParamCapability<PoolType_A, AssetType_A> {
            cap: asset_cap,
        });
    }

    /// Claim an asset in to pool
    public fun claim(account: &signer) {
        YieldFarming::claim<PoolType_A, Usdx, AssetType_A>(
            account, @alice, AssetType_A { value: 0 });
    }

    public fun stake(account: &signer, value: u128) {
        let asset_wrapper = YieldFarming::borrow_asset<PoolType_A, AssetType_A>(Signer::address_of(account));
        let (asset, _) = YieldFarming::borrow<PoolType_A, AssetType_A>(&mut asset_wrapper);
        asset.value = asset.value + value;
        YieldFarming::modify<PoolType_A, AssetType_A>(&mut asset_wrapper, asset.value);
        YieldFarming::stake<PoolType_A, Usdx, AssetType_A>(account, @alice, asset_wrapper);
    }

    public fun harvest(account: &signer) : Token::Token<Usdx> {
        YieldFarming::harvest_all<PoolType_A, Usdx, AssetType_A>(account, @alice)
    }

    public fun query_gov_token_amount(account: &signer) : u128 {
        YieldFarming::query_gov_token_amount<PoolType_A, Usdx, AssetType_A>(account, @alice)
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: alice
script {
    use 0x1::YieldFarming;

    /// Index test
    fun main(_account: signer) {
        let harvest_index = 100;
        let last_update_timestamp : u64 = 86395;
        let _asset_total_weight = 1000000000;

        let index_1 = YieldFarming::calculate_harvest_index(harvest_index, _asset_total_weight, last_update_timestamp, 2000000000);
        let withdraw_1 = YieldFarming::calculate_withdraw_amount(index_1, harvest_index, _asset_total_weight);
        assert((2000000000 * 5) == withdraw_1, 10001);

        // TODO: add more calculation for extreme scene ... 
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use 0x1::Account;
    use 0x1::Token;
    use 0x1::Math;
    use alice::TokenMock::{Usdx};

    fun init(account: signer) {
        let precision: u8 = 9; //STC precision is also 9.
        let scaling_factor = Math::pow(10, (precision as u64));
        let usdx_amount: u128 = 100000000 * scaling_factor;

        // Resister and mint Usdx
        Token::register_token<Usdx>(&account, precision);
        Account::do_accept_token<Usdx>(&account);
        let usdx_token = Token::mint<Usdx>(&account, usdx_amount);
        Account::deposit_to_self(&account, usdx_token);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use 0x1::Account;
    //use 0x1::Token;
    use 0x1::Math;
    use alice::TokenMock;

    fun init(account: signer) {
        let precision: u8 = 9; //Usdx precision is also 9.
        let scaling_factor = Math::pow(10, (precision as u64));
        let usdx_amount: u128 = 100 * scaling_factor;

        let tresury = Account::withdraw(&account, usdx_amount);
        TokenMock::initialize(&account, tresury);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 86410000

//! new-transaction
//! sender: joe
address alice = {{alice}};
address joe = {{joe}};
script {
    use alice::TokenMock;

    fun init(account: signer) {
        TokenMock::claim(&account);
        TokenMock::stake(&account, 10000);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 86430000

//! new-transaction
//! sender: joe
//address joe = {{joe}};
address alice = {{alice}};
script {
   use alice::TokenMock;
   use 0x1::Debug;

   fun init(account: signer) {
        let amount = TokenMock::query_gov_token_amount(&account);
        Debug::print(&amount);
        //assert(amount == 10, 1001);
   }
}
// check: EXECUTED

//! new-transaction
//! sender: joe
//address joe = {{joe}};
address alice = {{alice}};
script {
   use alice::TokenMock;
   use 0x1::Account;
   use 0x1::Token;
   use 0x1::Signer;

   fun init(account: signer) {
        Account::do_accept_token<TokenMock::Usdx>(&account);
        let token = TokenMock::harvest(&account);
        let token_balance = Token::value<TokenMock::Usdx>(&token);
        // Debug::print(&token_balance);

        assert(token_balance > 0, 1002);
        Account::deposit<TokenMock::Usdx>(Signer::address_of(&account), token);

        let amount = TokenMock::query_gov_token_amount(&account);
        assert(amount == 0, 1003);
   }
}