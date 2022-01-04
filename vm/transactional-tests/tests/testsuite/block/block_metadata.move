//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use Std::Block;
    use Std::Debug;

    fun get_parent_hash(_account: signer) {
        let hash = Block::get_parent_hash();
        Debug::print<vector<u8>>(&hash);
    }
}
// check: EXECUTED
