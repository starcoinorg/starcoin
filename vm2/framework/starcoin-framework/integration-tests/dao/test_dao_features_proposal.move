//# init -n dev

//# faucet --addr alice --amount 10000000000000000

//# faucet --addr bob

//# block --author 0x1 --timestamp 86400000

//# run --signers alice
script {
    use std::vector;
    use starcoin_framework::dao_features_proposal;

    fun proposal(alice: &signer) {
        let disable = vector::empty<u64>();
        vector::push_back(&mut disable, 1);

        dao_features_proposal::propose(alice, vector::empty<u64>(), disable, 3600000);
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 86460000

//# run --signers alice
script {
    use std::vector;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::primary_fungible_store;

    use starcoin_framework::dao;
    use starcoin_framework::dao_features_proposal;
    use starcoin_framework::starcoin_coin::STC;

    fun cast_vote_proposal(alice: &signer) {
        let disable = vector::empty<u64>();
        vector::push_back(&mut disable, 1);

        let balance = primary_fungible_store::withdraw(
            alice,
            starcoin_coin::get_stc_fa_metadata(),
            6370272400000001
        );
        dao::cast_vote<STC, dao_features_proposal::FeaturesUpdate>(
            alice,
            @alice,
            0,
            balance,
            true
        );
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 90070000

//# run --signers alice
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_std::debug;
    use starcoin_framework::dao_features_proposal;

    fun queue_proposal(_account: &signer) {
        debug::print(&std::string::utf8(b"queue_proposal | proposal state"));
        debug::print(&dao::proposal_state<STC, dao_features_proposal::FeaturesUpdate>(@alice, 0));
        dao::queue_proposal_action<STC, dao_features_proposal::FeaturesUpdate>(@alice, 0);
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 99000000

//# run --signers alice
script {
    use std::features;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::dao;
    use starcoin_std::debug;
    use starcoin_framework::dao_features_proposal;

    fun execute_proposal(_account: &signer) {
        debug::print(&std::string::utf8(b"execute_proposal | proposal state"));
        debug::print(&dao::proposal_state<STC, dao_features_proposal::FeaturesUpdate>(@alice, 0));
        assert!(features::is_enabled(1), 100);
        dao_features_proposal::execute(@alice, 0);
        assert!(!features::is_enabled(1), 101);
    }
}
// check: EXECUTED


//# run --signers core_resources
script {
    use std::features;
    use std::vector;
    use starcoin_framework::dao_features_proposal;

    fun test_execute_urgent(starcoin_association: &signer) {
        let enable = vector::empty<u64>();
        vector::push_back(&mut enable, 1);
        assert!(!features::is_enabled(1), 200);
        dao_features_proposal::execute_urgent(starcoin_association, enable, vector::empty<u64>());
        assert!(features::is_enabled(1), 201);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use std::features;
    use std::vector;
    use starcoin_framework::dao_features_proposal;

    fun test_execute_urgent(alice: &signer) {
        let disable = vector::empty<u64>();
        vector::push_back(&mut disable, 1);
        assert!(features::is_enabled(1), 200);
        dao_features_proposal::execute_urgent(alice, vector::empty<u64>(), disable);
        assert!(!features::is_enabled(1), 201);
    }
}
// check: "abort_code": "262145"