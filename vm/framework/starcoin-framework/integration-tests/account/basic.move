//# init -n dev

//# faucet --addr bob

//# faucet --addr alice --amount 0

//# faucet --addr default

//# publish
module default::holder {
    use starcoin_framework::signer;

    struct Hold<T> has key {
        x: T
    }

    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Hold<T> { x })
    }

    public fun get<T>(account: &signer): T
    acquires Hold {
        let Hold { x } = move_from<Hold<T>>(signer::address_of(account));
        x
    }
}


//# run --signers bob
//
// script {
//     use starcoin_framework::starcoin_coin::STC;
//     use starcoin_framework::account;
//
//     fun main(account: signer) {
//         let with_cap = account::offer_rotation_capability(&account);
//         account::pay_from_capability<STC>(&with_cap, @bob, 10, x"");
//         account::restore_withdraw_capability(with_cap);
//     }
// }
// check: EXECUTED
//
//
// //# run --signers bob
// script {
//     use starcoin_framework::account;
//
//     fun main(account: signer) {
//         let rot_cap = account::extract_key_rotation_capability(&account);
//         account::rotate_authentication_key_with_capability(&rot_cap, x"123abc");
//         account::restore_key_rotation_capability(rot_cap);
//     }
// }
//
// // check: ABORTED
// // check: 26119
//

//# run --signers default
// script {
//     use starcoin_framework::account;
//     use default::holder;
//
//     fun main(account: signer) {
//         Holder::hold(
//             &account,
//             account::extract_key_rotation_capability(&account)
//         );
//         Holder::hold(
//             &account,
//             account::extract_key_rotation_capability(&account)
//         );
//     }
// }
// // check: ABORTED
// // check: 26369
//
// //# run --signers default
// script {
//     use starcoin_framework::account;
//     use starcoin_framework::signer;
//
//     fun main(sender: signer) {
//         let cap = account::extract_key_rotation_capability(&sender);
//         assert!(
//             *account::key_rotation_capability_address(&cap) == signer::address_of(&sender), 0
//         );
//         account::restore_key_rotation_capability(cap);
//         let with_cap = account::extract_withdraw_capability(&sender);
//
//         assert!(
//             *account::withdraw_capability_address(&with_cap) == signer::address_of(&sender),
//             0
//         );
//         account::restore_withdraw_capability(with_cap);
//     }
// }
// // check: EXECUTED
//
//
// //# run --signers bob
//
// script {
//     use starcoin_framework::account;
//     use starcoin_framework::starcoin_coin::STC;
//
//     fun main(account: signer) {
//         let with_cap = account::extract_withdraw_capability(&account);
//         account::pay_from_capability<STC>(&with_cap, @alice, 10000, x"");
//         account::restore_withdraw_capability(with_cap);
//         assert!(coin::balance<STC>(@alice) == 10000, 60)
//     }
// }
// // check: EXECUTED
//
// //# run --signers default
// // test core address
//
// script {
//     use starcoin_framework::CoreAddresses;
//
//     fun main() {
//         assert!(CoreAddresses::VM_RESERVED_ADDRESS() == @0x0, 100);
//     }
// }
// // check: EXECUTED
//
// //# run --signers default
// script {
//     use starcoin_framework::account;
//     use starcoin_framework::starcoin_coin::STC;
//     use starcoin_framework::Authenticator;
//
//     fun main() {
//         let dummy_auth_key = x"91e941f5bc09a285705c092dd654b94a7a8e385f898968d4ecfba49609a13461";
//         let expected_address = Authenticator::derived_address(dummy_auth_key);
//         account::create_account_with_address<STC>(expected_address);
//         assert!(account::exists_at(expected_address), 1000);
//     }
// }
// // check: EXECUTED
//
//
// //# run --signers bob
// script {
//     use starcoin_framework::account;
//     use starcoin_framework::signer;
//
//     fun main(account: signer) {
//         let seq = account::sequence_number(signer::address_of(&account));
//         assert!(seq == 3, seq);
//     }
// }
// // check: EXECUTE
//

//# run --signers bob
script {
    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer) {
        coin::transfer<STC>(&account, @alice, 0);
    }
}
// check: EXECUTED