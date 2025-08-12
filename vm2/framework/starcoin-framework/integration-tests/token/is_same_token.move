//# init -n dev

//# faucet --addr alice


//# publish
module alice::fake_money {
    struct FakeMoney {}
}

//# run --signers alice
script {
    use starcoin_framework::stc_util;
    use starcoin_framework::starcoin_coin::{STC};
    use alice::fake_money;

    fun main() {
        assert!(stc_util::is_stc<STC>(), 1);
        assert!(!stc_util::is_stc<fake_money::FakeMoney>(), 2);
    }
}

// check: EXECUTED
