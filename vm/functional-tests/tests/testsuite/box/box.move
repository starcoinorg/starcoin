// Test Box
//! account: alice, 100000 0x1::STC::STC

module TestR {
    struct TestR has key, store {id: u64}

    public fun new(id: u64): TestR{
        TestR{
            id
        }
    }

    public fun id_of(r: &TestR):u64{
        r.id
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Box;
    use {{default}}::TestR;

    fun test_single(account: &signer) {
        let r = TestR::new(1);
        Box::put(account, r);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Box;
    use 0x1::Signer;
    use {{default}}::TestR::{Self, TestR};

    fun test_single(account: &signer) {
        let addr = Signer::address_of(account);
        assert(Box::exists_at<TestR>(addr), 1000);
        let r = Box::take<TestR>(account);
        assert(TestR::id_of(&r) == 1, 1001);
        assert(!Box::exists_at<TestR>(addr), 1002);
        Box::put(account, r);
        assert(Box::exists_at<TestR>(addr), 1003);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Box;
    use {{default}}::TestR;

    fun test_multi(account: &signer) {
        let r2 = TestR::new(2);
        Box::put(account, r2);
        let r3 = TestR::new(3);
        Box::put(account, r3);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Box;
    use 0x1::Signer;
    use {{default}}::TestR::{Self, TestR};

    fun test_single(account: &signer) {
        let addr = Signer::address_of(account);
        assert(Box::length<TestR>(addr) == 3, 1004);
        let r = Box::take<TestR>(account);
        assert(TestR::id_of(&r) == 3, 1005);
        Box::put(account, r);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Box;
    use 0x1::Signer;
    use 0x1::Vector;
    use {{default}}::TestR::{Self, TestR};

    fun test_multi(account: &signer) {
        let addr = Signer::address_of(account);
        assert(Box::length<TestR>(addr) == 3, 1006);
        let rv = Box::take_all<TestR>(account);
        assert(Vector::length<TestR>(&rv) == 3, 1007);
        assert(Box::length<TestR>(addr) == 0, 1008);
        Box::put_all(account, rv);
        assert(Box::length<TestR>(addr) == 3, 1009);

        let r = Box::take<TestR>(account);
        assert(TestR::id_of(&r) == 3, 1010);
        let rv_2 = Box::take_all<TestR>(account);
        assert(Vector::length<TestR>(&rv_2) == 2, 1012);
        assert(Box::length<TestR>(addr) == 0, 1013);
        Box::put(account, r);
        assert(Box::length<TestR>(addr) == 1, 1014);
        Box::put_all(account, rv_2);
        assert(Box::length<TestR>(addr) == 3, 1015);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Box;
    use 0x1::Signer;
    use {{default}}::TestR::{Self, TestR};

    fun test_take_invalid_state(account: &signer) {
        let addr = Signer::address_of(account);
        assert(Box::length<TestR>(addr) == 3, 1016);
        let r1 = Box::take<TestR>(account);
        assert(TestR::id_of(&r1) == 2, 1017);
        let r2 = Box::take<TestR>(account);
        assert(TestR::id_of(&r2) == 1, 1018);
        let r3 = Box::take<TestR>(account);
        assert(TestR::id_of(&r3) == 3, 1019);
        let invalid_r = Box::take<TestR>(account); //box is empty
        Box::put(account, r1);
        Box::put(account, r2);
        Box::put(account, r3);
        Box::put(account, invalid_r);
    }
}
// check: "Keep(ABORTED { code: 25857"

//! new-transaction
//! sender: alice
script {
    use 0x1::Box;
    use 0x1::Signer;
    use 0x1::Vector;
    use {{default}}::TestR::TestR;

    fun test_take_invalid_state(account: &signer) {
        let addr = Signer::address_of(account);
        assert(Box::length<TestR>(addr) == 3, 1020);
        let rv = Box::take_all<TestR>(account);
        assert(Vector::length<TestR>(&rv) == 3, 1021);
        let invalid_rv = Box::take_all<TestR>(account); //box is empty
        Box::put_all(account, invalid_rv);
        Box::put_all(account, rv);
    }
}
// check: "Keep(ABORTED { code: 25857"