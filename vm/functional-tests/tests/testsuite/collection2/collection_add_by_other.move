// Test Collection
//! account: alice, 100000 0x1::STC::STC
//! account: bob, 100000 0x1::STC::STC
address default={{default}};
module default::TestR {
    struct TestR has key, store {id: u64}

    public fun new(id: u64): TestR{
        TestR{
            id
        }
    }

    public fun id_of(r: &TestR):u64{
        r.id
    }

    public fun set_id(r: &mut TestR, new_id: u64){
            r.id = new_id;
    }

    public fun drop(r: TestR) {
        let TestR{id:_id} = r;
    }
}

//! new-transaction
//! sender: alice
address default={{default}};
script {
    use 0x1::Collection2;
    use default::TestR::{TestR};

    fun test_single(signer: signer) {
        Collection2::create_collection<TestR>(&signer, true, false);
    }
}

//! new-transaction
//! sender: bob
address default={{default}};
address alice = {{alice}};
script {
    use 0x1::Collection2;
    use default::TestR::{Self, TestR};

    fun test_add_by_other(signer: signer) {
        let c = Collection2::borrow_collection<TestR>(&signer, @alice);
        let r1 = TestR::new(1);
        Collection2::push_back(&mut c, r1);
        Collection2::return_collection(c);
    }
}

//! new-transaction
//! sender: alice
address default={{default}};
address alice = {{alice}};
script {
    use 0x1::Collection2;
    use default::TestR::{TestR};

    fun check(signer: signer) {
       let c = Collection2::borrow_collection<TestR>(&signer, @alice);
       assert(Collection2::length(&c) == 1, 1000);
        Collection2::return_collection(c);
    }
}

//! new-transaction
//! sender: bob
address default={{default}};
address alice = {{alice}};
address bob = {{bob}};
script {
    use 0x1::Collection2;
    use default::TestR::{Self, TestR};

    fun test_mut_by_other(signer: signer) {
        let c = Collection2::borrow_collection<TestR>(&signer, @alice);
        let r1 = Collection2::borrow_mut(&mut c, 0);
        TestR::set_id(r1, 2);
        Collection2::return_collection(c);
    }
}

//mut by other at here failed.
// check: "Keep(ABORTED { code: 26370"