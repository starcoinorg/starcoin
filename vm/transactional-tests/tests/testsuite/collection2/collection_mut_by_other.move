
//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr default

//# publish

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


//# run --signers alice

script {
    use StarcoinFramework::Collection2;
    use default::TestR::{TestR};

    fun test_single(signer: signer) {
        Collection2::create_collection<TestR>(&signer, false, true);
    }
}


//# run --signers bob


script {
    use StarcoinFramework::Collection2;
    use default::TestR::{Self, TestR};

    fun test_add_by_other(signer: signer) {
        let c = Collection2::borrow_collection<TestR>(&signer, @alice);
        let r1 = TestR::new(1);
        Collection2::push_back(&mut c, r1);
        Collection2::return_collection(c);
    }
}

//add by other at here failed.
// check: "Keep(ABORTED { code: 26114"


//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use default::TestR::{Self,TestR};

    fun add_by_owner(signer: signer) {
       let c = Collection2::borrow_collection<TestR>(&signer, @alice);
       let r1 = TestR::new(1);
       Collection2::push_back(&mut c, r1);
       Collection2::return_collection(c);
    }
}


//# run --signers bob


script {
    use StarcoinFramework::Collection2;
    use default::TestR::{Self, TestR};

    fun test_mut_by_other(signer: signer) {
        let c = Collection2::borrow_collection<TestR>(&signer, @alice);
        let r1 = Collection2::borrow_mut(&mut c, 0);
        TestR::set_id(r1, 2);
        Collection2::return_collection(c);
    }
}


//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use default::TestR::{Self,TestR};

    fun check(signer: signer) {
       let c = Collection2::borrow_collection<TestR>(&signer, @alice);
       let r1 = Collection2::borrow(&c, 0);
       assert!(TestR::id_of(r1) == 2, 1001);
       Collection2::return_collection(c);
    }
}