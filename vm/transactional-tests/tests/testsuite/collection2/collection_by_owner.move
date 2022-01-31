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

    public fun drop(r: TestR) {
        let TestR{id:_id} = r;
    }
}


//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use StarcoinFramework::Signer;
    use default::TestR::{Self,TestR};

    fun test_single(signer: signer) {
        let r = TestR::new(1);
        Collection2::create_collection<TestR>(&signer, false, false);
        Collection2::put(&signer, Signer::address_of(&signer), r);
    }
}


//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use StarcoinFramework::Signer;
    use default::TestR::{Self, TestR};

    fun test_single(signer: signer) {
        let addr = Signer::address_of(&signer);
        let c = Collection2::borrow_collection<TestR>(&signer,addr);
        let r = Collection2::pop_back(&mut c);
        assert!(TestR::id_of(&r) == 1, 1001);
        Collection2::append(&mut c, r);
        Collection2::return_collection(c);
    }
}


//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use StarcoinFramework::Signer;
    use default::TestR::{Self,TestR};

    fun test_multi(signer: signer) {
        let addr = Signer::address_of(&signer);
        let c = Collection2::borrow_collection<TestR>(&signer, addr);
        let r2 = TestR::new(2);
        Collection2::append(&mut c, r2);
        let r3 = TestR::new(3);
        Collection2::append(&mut c, r3);
        Collection2::return_collection(c);
    }
}


//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use StarcoinFramework::Signer;
    use default::TestR::{Self, TestR};

    fun test_borrow_by_owner(signer: signer) {
        let addr = Signer::address_of(&signer);
        let c = Collection2::borrow_collection<TestR>(&signer, addr);
        let c_len = Collection2::length<TestR>(&c);
        assert!(c_len == 3, 1004);
        let i = 0;

        while (i < c_len) {
            let r = Collection2::borrow(&c, i);
            let id = TestR::id_of(r);
            assert!(id == i + 1, 1005);
            i = i + 1;
        };
        Collection2::return_collection(c);
    }
}


//# run --signers bob


script {
    use StarcoinFramework::Collection2;
    use default::TestR::{Self, TestR};

    fun test_borrow_by_other(signer: signer) {
        let c = Collection2::borrow_collection<TestR>(&signer, @alice);
        let r = Collection2::borrow(&c, 0);
        let id = TestR::id_of(r);
        assert!(id == 1, 1006);
        Collection2::return_collection(c);
    }
}


//# run --signers bob


script {
    use StarcoinFramework::Collection2;
    use default::TestR::{Self, TestR};

    fun test_remove_by_other(signer: signer) {
        let c = Collection2::borrow_collection<TestR>(&signer, @alice);
        let r = Collection2::remove<TestR>(&mut c, 0);
        TestR::drop(r);
        Collection2::return_collection(c);
    }
}

// check: "Keep(ABORTED { code: 26626"



//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use StarcoinFramework::Signer;
    use default::TestR::{Self, TestR};

    fun test_remove_by_owner(signer: signer) {
        let addr = Signer::address_of(&signer);
        let c = Collection2::borrow_collection<TestR>(&signer, addr);

        let c_len = Collection2::length<TestR>(&c);
        let i = 0;
        //remove all items.
        while (i < c_len) {
            let r = Collection2::remove(&mut c, 0);
            TestR::drop(r);
            i = i + 1;
        };
        Collection2::return_collection(c);
        Collection2::destroy_collection<TestR>(&signer);
        assert!(!Collection2::exists_at<TestR>(addr), 1007)
    }
}


//# run --signers alice


script {
    use StarcoinFramework::Collection2;
    use StarcoinFramework::Signer;
    use default::TestR::{TestR};

    fun test_remove_by_owner(signer: signer) {
        let addr = Signer::address_of(&signer);
        let c = Collection2::borrow_collection<TestR>(&signer, addr);
        Collection2::return_collection(c);
    }
}

// check: "Keep(ABORTED { code: 25857"