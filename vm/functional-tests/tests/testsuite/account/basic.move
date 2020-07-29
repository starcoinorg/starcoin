//! account: bob, 10000 0x1::STC::STC
//! account: alice, 0 0x1::STC::STC

module Holder {
    use 0x1::Signer;

    resource struct Hold<T> {
        x: T
    }

    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Hold<T>{x})
    }

    public fun get<T>(account: &signer): T
    acquires Hold {
        let Hold {x} = move_from<Hold<T>>(Signer::address_of(account));
        x
    }
}


//! new-transaction
//! sender: bob
script {
    use 0x1::STC::STC;
    use 0x1::Account;
    fun main(account: &signer) {
        let with_cap = Account::extract_withdraw_capability(account);
        Account::pay_from_capability<STC>({{bob}}, &with_cap, 10, x"", x"");
        Account::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    fun main(account: &signer) {
        let rot_cap = Account::extract_key_rotation_capability(account);
        Account::rotate_authentication_key(&rot_cap, x"123abc");
        Account::restore_key_rotation_capability(rot_cap);
    }
}

// check: ABORTED
// check: 23

//! new-transaction
script {
    use 0x1::Account;
    use {{default}}::Holder;
    fun main(account: &signer) {
        Holder::hold(
            account,
            Account::extract_key_rotation_capability(account)
        );
        Holder::hold(
            account,
            Account::extract_key_rotation_capability(account)
        );
    }
}
// check: ABORTED
// check: 24

//! new-transaction
script {
    use 0x1::Account;
    use 0x1::Signer;
    fun main(sender: &signer) {
        let cap = Account::extract_key_rotation_capability(sender);
        assert(
            *Account::key_rotation_capability_address(&cap) == Signer::address_of(sender), 0
        );
        Account::restore_key_rotation_capability(cap);
        let with_cap = Account::extract_withdraw_capability(sender);

        assert(
            *Account::withdraw_capability_address(&with_cap) == Signer::address_of(sender),
            0
        );
        Account::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    fun main(account: &signer) {
        let with_cap = Account::extract_withdraw_capability(account);
        Account::pay_from_capability<STC>({{alice}}, &with_cap, 10000, x"", x"");
        Account::restore_withdraw_capability(with_cap);
        assert(Account::balance<STC>({{alice}}) == 10000, 60)
    }
}
// check: EXECUTED

// test core address
//! new-transaction
script {
    use 0x1::CoreAddresses;
    fun main() {
       assert(CoreAddresses::VM_RESERVED_ADDRESS() == 0x0, 100);
    }
}
// check: EXECUTED
