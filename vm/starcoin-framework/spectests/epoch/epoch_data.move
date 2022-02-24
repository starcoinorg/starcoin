//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use StarcoinFramework::Epoch;

    fun epoch_data() {
        // default value should be consistent with genesis config
        let default_block_gas_limit = 50000000*10;
        let default_block_time_target = 10000;
        let default_number = 0;
        let default_start_block_number = 0;
        let default_end_block_number = 24 * 2;
        let default_start_time = 0;
        let default_total_gas = 0;
        let default_uncles = 0;

        let block_gas_limit = Epoch::block_gas_limit();
        let block_time_target = Epoch::block_time_target();
        let number = Epoch::number();
        let start_block_number = Epoch::start_block_number();
        let end_block_number = Epoch::end_block_number();
        let start_time = Epoch::start_time();
        let total_gas = Epoch::total_gas();
        let uncles = Epoch::uncles();

        assert!(block_gas_limit == default_block_gas_limit, 101);
        assert!(block_time_target == default_block_time_target, 102);
        assert!(number == default_number, 103);
        assert!(start_block_number == default_start_block_number, 104);
        assert!(end_block_number == default_end_block_number, 105);
        assert!(start_time == default_start_time, 106);
        assert!(total_gas == default_total_gas, 107);
        assert!(uncles == default_uncles, 108);
    }
}