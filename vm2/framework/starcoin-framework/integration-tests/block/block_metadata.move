//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::stc_block;
    use starcoin_framework::debug;

    fun get_parent_hash(_account: signer) {
        let hash = stc_block::get_parent_hash();
        debug::print<vector<u8>>(&hash);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::stc_block;
    use starcoin_framework::debug;

    fun get_parents_hash(_account: signer) {
        let hash = stc_block::get_parents_hash();
        debug::print<vector<u8>>(&hash);
    }
}
// check: EXECUTED
