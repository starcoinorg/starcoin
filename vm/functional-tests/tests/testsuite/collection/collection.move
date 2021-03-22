// Test Collection
//! account: alice, 100000 0x1::STC::STC
//! account: bob, 100000 0x1::STC::STC

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

    public fun drop(r: TestR) {
        let TestR{id:_id} = r;
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Collection;
    use {{default}}::TestR;

    fun test_single(account: &signer) {
        let r = TestR::new(1);
        Collection::put(account, r);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Collection;
    use 0x1::Signer;
    use {{default}}::TestR::{Self, TestR};

    fun test_single(account: &signer) {
        let addr = Signer::address_of(account);
        assert(Collection::has<TestR>(addr), 1000);
        let c = Collection::borrow_collection<TestR>(addr);
        let r = Collection::pop_back(account, &mut c);
        assert(TestR::id_of(&r) == 1, 1001);
        // the has method will return true when collection is borrowed.
        assert(Collection::has<TestR>(addr), 1002);
        Collection::append(account, &mut c, r);
        Collection::return_collection(c);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Collection;
    use 0x1::Signer;
    use {{default}}::TestR::{Self,TestR};

    fun test_multi(account: &signer) {
        let addr = Signer::address_of(account);
        let c = Collection::borrow_collection<TestR>(addr);
        let r2 = TestR::new(2);
        Collection::append(account, &mut c, r2);
        let r3 = TestR::new(3);
        Collection::append(account, &mut c, r3);
        Collection::return_collection(c);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Collection;
    use 0x1::Signer;
    use {{default}}::TestR::{Self, TestR};

    fun test_borrow_by_owner(account: &signer) {
        let addr = Signer::address_of(account);
        let c = Collection::borrow_collection<TestR>(addr);
        let c_len = Collection::length<TestR>(&c);
        assert(c_len == 3, 1004);
        let i = 0;

        while (i < c_len) {
            let r = Collection::borrow(&c, i);
            let id = TestR::id_of(r);
            assert(id == i + 1, 1005);
            i = i + 1;
        };
        Collection::return_collection(c);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Collection;
    use {{default}}::TestR::{Self, TestR};

    fun test_borrow_by_other(_account: &signer) {
        let c = Collection::borrow_collection<TestR>({{alice}});
        let r = Collection::borrow(&c, 0);
        let id = TestR::id_of(r);
        assert(id == 1, 1006);
        Collection::return_collection(c);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Collection;
    use {{default}}::TestR::{Self, TestR};

    fun test_remove_by_other(account: &signer) {
        let c = Collection::borrow_collection<TestR>({{alice}});
        let r = Collection::remove<TestR>(account, &mut c, 0);
        TestR::drop(r);
        Collection::return_collection(c);
    }
}

// check: "Keep(ABORTED { code: 26114"


//! new-transaction
//! sender: alice
script {
    use 0x1::Collection;
    use 0x1::Signer;
    use {{default}}::TestR::{Self, TestR};

    fun test_remove_by_owner(account: &signer) {
        let addr = Signer::address_of(account);
        let c = Collection::borrow_collection<TestR>(addr);

        let c_len = Collection::length<TestR>(&c);
        let i = 0;
        //remove all items.
        while (i < c_len) {
            let r = Collection::remove(account, &mut c, 0);
            TestR::drop(r);
            i = i + 1;
        };
        Collection::return_collection(c);
        assert(!Collection::has<TestR>(addr), 1007)
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Collection;
    use 0x1::Signer;
    use {{default}}::TestR::{TestR};

    fun test_remove_by_owner(account: &signer) {
        let addr = Signer::address_of(account);
        let c = Collection::borrow_collection<TestR>(addr);
        Collection::return_collection(c);
    }
}

// check: "Keep(ABORTED { code: 25857"