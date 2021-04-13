//! account: alice

//! sender: alice
script {
    use 0x1::Block;
    use 0x1::Debug;

    fun get_parent_hash(_account: signer) {
        let hash = Block::get_parent_hash();
        Debug::print<vector<u8>>(&hash);
    }
}
// check: EXECUTED
