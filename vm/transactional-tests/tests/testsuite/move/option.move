//# init -n dev

//# faucet --addr creator

//# run --signers creator
// Tests for the non-aborting behavior of Option functions
script {
use StarcoinFramework::Option;

fun main() {
    let some = Option::some(5);
    let none = Option::none<u64>();
    // borrow/get_with_default
    assert!(*Option::borrow_with_default(&some, &7) == 5, 8007);
    assert!(*Option::borrow_with_default(&none, &7) == 7, 8008);
    assert!(Option::get_with_default(&some, 7) == 5, 8009);
    assert!(Option::get_with_default(&none, 7) == 7, 8010);

    Option::swap(&mut some, 1);
    assert!(*Option::borrow(&some) == 1, 8011);

    // contains/immutable borrowing
    assert!(Option::contains(&some, &1), 8012);
    assert!(!Option::contains(&none, &5), 8013);

   // destroy_with_default, destroy_some, destroy_none
   assert!(Option::destroy_with_default(Option::none<u64>(), 4) == 4, 8014);
   assert!(Option::destroy_with_default(Option::some(4), 5) == 4, 8015);
   assert!(Option::destroy_some(Option::some(4)) == 4, 8016);
   Option::destroy_none(Option::none<u64>());

    // fill
    Option::fill(&mut none, 3);
    let three = none;
    assert!(Option::is_some(&three), 8017);
    assert!(*Option::borrow(&three) == 3, 8018);

   // letting an Option<u64> go out of scope is also ok
   let _ = Option::some(7);
    }
  }

// check: EXECUTED
