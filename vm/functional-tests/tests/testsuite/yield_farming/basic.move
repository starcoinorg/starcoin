//! account: alice, 100000000000000000 0x1::STC::STC
//! account: bob
//! account: cindy
//! account: davied
//! account: joe

//! sender: alice
address alice = {{alice}};
module alice::YieldFarmingWarpper {
    use 0x1::Token;
    use 0x1::YieldFarming;
    use 0x1::Account;
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

    public fun stake(account: &signer, value: u128) {
        YieldFarming::stake<PoolType_A, Usdx, AssetType_A>(account, @alice, AssetType_A { value }, value);
    }

    public fun unstake(account: &signer) : (u128, u128) {
        let (asset, token) = YieldFarming::unstake<PoolType_A, Usdx, AssetType_A>(account, @alice);
        let token_val = Token::value<Usdx>(&token);
        Account::deposit<Usdx>(Signer::address_of(account), token);
        (asset.value, token_val)
    }

    public fun harvest(account: &signer) : Token::Token<Usdx> {
        YieldFarming::harvest<PoolType_A, Usdx, AssetType_A>(account, @alice, 0)
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
    use 0x1::Timestamp;
    use 0x1::Debug;

    /// Index test
    fun main(_account: signer) {
        let harvest_index = 100;
        let last_update_timestamp : u64 = 86395;
        let _asset_total_weight = 1000000000;

        Debug::print(&last_update_timestamp);
        Debug::print(&Timestamp::now_seconds());

        let index_1 = YieldFarming::calculate_harvest_index(
            harvest_index,
            _asset_total_weight,
            last_update_timestamp, 
            Timestamp::now_seconds(), 2000000000);
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
    use alice::YieldFarmingWarpper::{Usdx};

    /// Initial reward token, registered and mint it
    fun main(account: signer) {
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
    use 0x1::Math;
    use alice::YieldFarmingWarpper;

    /// Inital a treasury into yield farming
    fun init(account: signer) {
        let precision: u8 = 9; //Usdx precision is also 9.
        let scaling_factor = Math::pow(10, (precision as u64));
        let usdx_amount: u128 = 100 * scaling_factor;

        let tresury = Account::withdraw(&account, usdx_amount);
        YieldFarmingWarpper::initialize(&account, tresury);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
address alice = {{alice}};
address bob = {{bob}};
script {
    use alice::YieldFarmingWarpper::{Usdx, Self};
    use 0x1::Account;
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Debug;

    /// 1. First stake, check whether first rewards has been executed.
    fun main(account: signer) {
        Account::do_accept_token<Usdx>(&account);
        YieldFarmingWarpper::stake(&account, 100000000);
        
        let token = YieldFarmingWarpper::harvest(&account);
        let _amount = Token::value<Usdx>(&token);
        Debug::print(&_amount);
        // assert(amount == 10000000000, 10002);
        Account::deposit<Usdx>(Signer::address_of(&account), token);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 86420000

//! new-transaction
//! sender: cindy
address alice = {{alice}};
address bob = {{bob}};
script {
    use alice::YieldFarmingWarpper::{Usdx, Self};
    use 0x1::Account;

    /// 2. Cindy joined and staking some asset
    fun init(account: signer) {
        Account::do_accept_token<Usdx>(&account);
        YieldFarmingWarpper::stake(&account, 100000000);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 86430000

//! new-transaction
//! sender: cindy
address alice = {{alice}};
address bob = {{bob}};
script {
    use alice::YieldFarmingWarpper;
    use 0x1::Debug;

    /// 3. Cindy harvest after 20 seconds, checking whether has rewards.
    fun init(account: signer) {
        let amount00 = YieldFarmingWarpper::query_gov_token_amount(&account);
        Debug::print(&amount00);
        // assert(amount00 == 0, 10004);
        assert(amount00 > 0, 10004);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 86440000

//! new-transaction
//! sender: cindy
address alice = {{alice}};
address bob = {{bob}};
script {
    use alice::YieldFarmingWarpper::{Usdx, Self};
    use 0x1::Account;
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Debug;

    /// 4. Cindy harvest after 40 seconds, checking whether has rewards.
    fun init(account: signer) {
        let amount00 = YieldFarmingWarpper::query_gov_token_amount(&account);
        Debug::print(&amount00);

        let token = YieldFarmingWarpper::harvest(&account);
        let amount1 = Token::value<Usdx>(&token);
        Debug::print(&amount1);
        assert(amount1 > 0, 10005);
        // assert(amount1 == 20000000000, 10004);
        Account::deposit<Usdx>(Signer::address_of(&account), token);
    }
}
// check: EXECUTED

