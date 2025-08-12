//# init -n dev

//# faucet --addr creator

//# run --signers creator
// Tests for the non-aborting behavior of option functions
script {
    use std::option;

    fun main() {
        let some = option::some(5);
        let none = option::none<u64>();
        // borrow/get_with_default
        assert!(*option::borrow_with_default(&some, &7) == 5, 8007);
        assert!(*option::borrow_with_default(&none, &7) == 7, 8008);
        assert!(option::get_with_default(&some, 7) == 5, 8009);
        assert!(option::get_with_default(&none, 7) == 7, 8010);

        option::swap(&mut some, 1);
        assert!(*option::borrow(&some) == 1, 8011);

        // contains/immutable borrowing
        assert!(option::contains(&some, &1), 8012);
        assert!(!option::contains(&none, &5), 8013);

        // destroy_with_default, destroy_some, destroy_none
        assert!(option::destroy_with_default(option::none<u64>(), 4) == 4, 8014);
        assert!(option::destroy_with_default(option::some(4), 5) == 4, 8015);
        assert!(option::destroy_some(option::some(4)) == 4, 8016);
        option::destroy_none(option::none<u64>());

        // fill
        option::fill(&mut none, 3);
        let three = none;
        assert!(option::is_some(&three), 8017);
        assert!(*option::borrow(&three) == 3, 8018);

        // letting an option<u64> go out of scope is also ok
        let _ = option::some(7);
    }
}

// check: EXECUTED
