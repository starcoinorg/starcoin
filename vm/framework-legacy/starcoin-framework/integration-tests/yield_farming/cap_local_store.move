//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# faucet --addr bob

//# faucet --addr cindy

//# faucet --addr davied

//# faucet --addr joe

//# publish
module alice::YieldFarmingWarpper {
    use StarcoinFramework::Token;
    use StarcoinFramework::Account;
    use StarcoinFramework::Signer;
    use StarcoinFramework::YieldFarmingV2;

    struct Usdx has copy, drop, store {}

    struct PoolType_A has copy, drop, store {}

    struct AssetType_A has copy, drop, store { value: u128 }

    struct GovModfiyParamCapability has key, store {
        cap: YieldFarmingV2::ParameterModifyCapability<PoolType_A, AssetType_A>,
    }

    public fun initialize(signer: &signer, treasury: Token::Token<Usdx>) {
        YieldFarmingV2::initialize<PoolType_A, Usdx>(signer, treasury);
        let asset_cap = YieldFarmingV2::add_asset<PoolType_A, AssetType_A>(signer, 1000000000, 0);
        move_to(signer, GovModfiyParamCapability {
            cap: asset_cap,
        });
    }

    public fun stake(signer: &signer, value: u128) acquires GovModfiyParamCapability {
        let cap = borrow_global_mut<GovModfiyParamCapability>(@alice);
        YieldFarmingV2::stake<PoolType_A, Usdx, AssetType_A>(
            signer, @alice, AssetType_A { value }, value, &cap.cap);
    }

    public fun unstake(signer: &signer): (u128, u128) {
        let (asset, token) = YieldFarmingV2::unstake<PoolType_A, Usdx, AssetType_A>(signer, @alice);
        let token_val = Token::value<Usdx>(&token);
        Account::deposit<Usdx>(Signer::address_of(signer), token);
        (asset.value, token_val)
    }

    public fun harvest(account: &signer): Token::Token<Usdx> {
        YieldFarmingV2::harvest<PoolType_A, Usdx, AssetType_A>(account, @alice, 0)
    }

    public fun query_gov_token_amount(account: address): u128 {
        YieldFarmingV2::query_gov_token_amount<PoolType_A, Usdx, AssetType_A>(account, @alice)
    }

    public fun modify_parameter(release_per_second: u128) acquires GovModfiyParamCapability {
        let cap = borrow_global_mut<GovModfiyParamCapability>(@alice);
        YieldFarmingV2::modify_parameter<PoolType_A, Usdx, AssetType_A>(&cap.cap, @alice, release_per_second, true);
    }
}

//# block --author 0x1 --timestamp 86400000

//# run --signers alice
script {
    use StarcoinFramework::YieldFarmingV2;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Debug;

    /// Index test
    fun main(_account: signer) {
        let harvest_index = 100;
        let last_update_timestamp: u64 = 86395;
        let _asset_total_weight = 1000000000;

        let index_1 = YieldFarmingV2::calculate_harvest_index(
            harvest_index,
            _asset_total_weight,
            last_update_timestamp,
            Timestamp::now_seconds(), 2000000000);
        let withdraw_1 = YieldFarmingV2::calculate_withdraw_amount(index_1, harvest_index, _asset_total_weight);
        assert!((2000000000 * 5) == withdraw_1, 1001);

        // Denominator bigger than numberator
        let index_2 = YieldFarmingV2::calculate_harvest_index(0, 100000000000000, 0, 5, 10000000);
        let amount_2 = YieldFarmingV2::calculate_withdraw_amount(index_2, 0, 40000000000);
        Debug::print(&index_2);
        Debug::print(&amount_2);
        assert!(index_2 > 0, 1002);
        assert!(amount_2 > 0, 1003);
        //let withdraw_1 = YieldFarmingV2::calculate_withdraw_amount(index_1, harvest_index, _asset_total_weight);
        //assert!((2000000000 * 5) == withdraw_1, 10001);
    }
}



//# run --signers alice
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::Token;
    use StarcoinFramework::Math;
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

//# run --signers alice
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::Math;
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

//# run --signers bob
script {
    use alice::YieldFarmingWarpper::{Usdx, Self};
    use StarcoinFramework::Account;
    use StarcoinFramework::Token;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Debug;

    /// 1. First stake, check whether first rewards has been executed.
    fun main(account: signer) {
        Account::do_accept_token<Usdx>(&account);
        YieldFarmingWarpper::stake(&account, 100000000);

        let token = YieldFarmingWarpper::harvest(&account);
        let _amount = Token::value<Usdx>(&token);
        Debug::print(&_amount);
        // assert!(amount == 10000000000, 10002);
        Account::deposit<Usdx>(Signer::address_of(&account), token);
    }
}


//# block --author 0x1 --timestamp 86420000


//# run --signers cindy
script {
    use alice::YieldFarmingWarpper::{Usdx, Self};
    use StarcoinFramework::Account;

    /// 2. Cindy joined and staking some asset
    fun init(account: signer) {
        Account::do_accept_token<Usdx>(&account);
        YieldFarmingWarpper::stake(&account, 100000000);
    }
}
//# block --author 0x1 --timestamp 86430000

//# run --signers cindy
script {
    use alice::YieldFarmingWarpper;
    use StarcoinFramework::Debug;
    use StarcoinFramework::Signer;

    /// 3. Cindy harvest after 20 seconds, checking whether has rewards.
    fun init(account: signer) {
        let amount00 = YieldFarmingWarpper::query_gov_token_amount(Signer::address_of(&account));
        Debug::print(&amount00);
        // assert!(amount00 == 0, 10004);
        assert!(amount00 > 0, 10004);
    }
}

//# block --author 0x1 --timestamp 86440000

//# run --signers cindy
script {
    use alice::YieldFarmingWarpper::{Usdx, Self};
    use StarcoinFramework::Account;
    use StarcoinFramework::Token;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Debug;

    /// 4. Cindy harvest after 40 seconds, checking whether has rewards.
    fun init(account: signer) {
        let amount00 = YieldFarmingWarpper::query_gov_token_amount(Signer::address_of(&account));
        Debug::print(&amount00);

        let token = YieldFarmingWarpper::harvest(&account);
        let amount1 = Token::value<Usdx>(&token);
        Debug::print(&amount1);
        assert!(amount1 > 0, 10005);
        // assert!(amount1 == 20000000000, 10004);
        Account::deposit<Usdx>(Signer::address_of(&account), token);
    }
}

//# run --signers alice
script {
    use alice::YieldFarmingWarpper::{Self};

    /// modify parameter test
    fun init(_signer: signer) {
        YieldFarmingWarpper::modify_parameter(100000000);
    }
}
