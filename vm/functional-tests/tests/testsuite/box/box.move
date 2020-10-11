// Test Box
//! account: alice, 100000 0x1::STC::STC

module TestR {
    resource struct TestR{id: u64}

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
