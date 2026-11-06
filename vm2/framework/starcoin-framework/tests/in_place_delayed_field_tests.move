#[test_only]
module starcoin_framework::in_place_delayed_field_tests {
    use std::signer;
    use starcoin_framework::aggregator_v2;

    struct DelayedFieldHolder has key {
        marker: u64,
        amount: aggregator_v2::Aggregator<u64>,
    }

    fun publish_holder(account: &signer, marker: u64, start: u64) {
        move_to(
            account,
            DelayedFieldHolder {
                marker,
                amount: aggregator_v2::create_unbounded_aggregator_with_value(start),
            },
        );
    }

    fun add_amount_only(addr: address, delta: u64) acquires DelayedFieldHolder {
        let holder = borrow_global_mut<DelayedFieldHolder>(addr);
        aggregator_v2::add(&mut holder.amount, delta);
    }

    fun read_amount(addr: address): u64 acquires DelayedFieldHolder {
        let holder = borrow_global<DelayedFieldHolder>(addr);
        aggregator_v2::read(&holder.amount)
    }

    fun read_marker(addr: address): u64 acquires DelayedFieldHolder {
        let holder = borrow_global<DelayedFieldHolder>(addr);
        holder.marker
    }

    #[test(account = @starcoin_framework)]
    fun test_write_delayed_field_in_place(account: signer) acquires DelayedFieldHolder {
        let addr = signer::address_of(&account);
        publish_holder(&account, 7, 10);

        // Only mutate delayed field (aggregator) in Move layer.
        add_amount_only(addr, 2);
        add_amount_only(addr, 5);

        assert!(read_marker(addr) == 7, 0);
        assert!(read_amount(addr) == 17, 1);
    }
}
