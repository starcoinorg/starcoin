address 0x1{
module SubsidyConfig {

    use 0x1::Signer;

    resource struct T {
        subsidy_halving_interval: u64,
        subsidy_base: u64,
        subsidy_delay: u64,
        set: bool,
    }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == 0x6d696e74, 6101);
        assert(!exists<T>(Signer::address_of(account)), 6102);

        move_to<T>(account, T {
            subsidy_halving_interval: 0,
            subsidy_base: 0,
            subsidy_delay: 0,
            set: false,
        });
    }

    public fun subsidy(account: &signer, halving: u64, subsidy: u64, delay: u64) acquires T {
        assert(Signer::address_of(account) == 0x6d696e74, 6103);
        assert(exists<T>(Signer::address_of(account)), 6104);

        let consensus_account = borrow_global_mut<T>(Signer::address_of(account));

        assert(!(consensus_account.set), 6105);

        assert(halving > 0, 6106);
        assert(subsidy > 0, 6107);
        assert(delay > 0, 6108);

        consensus_account.subsidy_halving_interval = halving;
        consensus_account.subsidy_base = subsidy;
        consensus_account.subsidy_delay = delay;

        consensus_account.set = true;
    }

    public fun subsidy_coin(height:u64): u64 acquires T {
        assert(right_conf(), 6109);

        let halving = subsidy_halving();
        let subsidy = subsidy_base();
        let times = height / halving;
        let index = 0;

        while (index < times) {
            if (subsidy == 0) {
                break
            };
            subsidy = subsidy / 2;
            index = index + 1;
        };

        subsidy
    }

    public fun subsidy_halving(): u64 acquires T {
        borrow_global<T>(0x6d696e74).subsidy_halving_interval
    }

    public fun subsidy_base(): u64 acquires T {
        borrow_global<T>(0x6d696e74).subsidy_base
    }

    public fun subsidy_delay(): u64 acquires T {
        borrow_global<T>(0x6d696e74).subsidy_delay
    }

    public fun already_set(): bool acquires T {
        borrow_global<T>(0x6d696e74).set
    }

    public fun right_conf():bool acquires T {
        if ((subsidy_halving() <= 0) || (subsidy_base() <= 0) || (subsidy_delay() <= 0) || !already_set()){
            false
        } else {
            true
        }
    }
}
}