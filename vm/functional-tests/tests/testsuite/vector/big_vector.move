//! account: alice
//! account: bob

//! sender: alice
address alice = {{alice}};
module alice::BigVectorTest {
    use 0x1::Vector;
    use 0x1::Signer;

    struct Element<T> has key, store {
        addr: address,
        value: T
    }

    struct BigVector<T> has key, store {
        vec: vector<Element<T>>
    }

    public fun init(account: &signer) {
        let vec = Vector::empty<Element<u64>>();
        let index = 0;
        while (index < 5000) {
            let element = Element<u64> {
                addr: @0x1,
                value: index,
            };
            Vector::push_back(&mut vec, element);
            index = index + 1;
        };

        move_to<BigVector<u64>>(account, BigVector<u64> {vec});
        //assert(Vector::contains<u64>(&vec, &(@0x1, 99)) == true, 1001);
    }

    // append num of elements to vector
    public fun append(account: &signer, num: u64) acquires BigVector {
        let vector = &mut borrow_global_mut<BigVector<u64>>(Signer::address_of(account)).vec;
        let index = Vector::length<Element<u64>>(vector);
        let total = index + num;
        while (index < total) {
            let element = Element<u64> {
                addr: @0x1,
                value: index,
            };
            Vector::push_back(vector, element);
            index = index + 1;
        }
    }

    public fun value_of(account: &signer, index: u64): u64 acquires BigVector {
        let vector = &borrow_global<BigVector<u64>>(Signer::address_of(account)).vec;
        Vector::borrow<Element<u64>>(vector, index).value
    }

    public fun remove(account: &signer, index: u64) acquires BigVector {
        let vector = &mut borrow_global_mut<BigVector<u64>>(Signer::address_of(account)).vec;
        let Element<u64> {addr: _, value: _} = Vector::remove<Element<u64>>(vector, index);
    }

    public fun index_of(account: &signer, addr: address, value: u64): (bool, u64) acquires BigVector {
        let vector = &borrow_global<BigVector<u64>>(Signer::address_of(account)).vec;
        let element = Element<u64> {
            addr,
            value,
        };
        let (has, index) = Vector::index_of(vector, &element);
        let Element<u64> {addr: _, value: _} = element;
        (has, index)
    }


}

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::init(&account);
        assert(BigVectorTest::value_of(&account, 4999) == 4999, 101);
    }
}
// check: gas_used
// check: 28542057
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::append(&account, 1);
        assert(BigVectorTest::value_of(&account, 5000) == 5000, 102);
    }
}
// check: gas_used
// check: 38582
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
// appand 5000 elements "5000, 5001, ... 10000" to the vector
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::append(&account, 5000);
        assert(BigVectorTest::value_of(&account, 10000) == 10000, 103);
    }
}
// check: gas_used
// check: 23153958
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::append(&account, 1);
        assert(BigVectorTest::value_of(&account, 10001) == 10001, 104);
    }
}
// check: gas_used
// check: 38582
// check: "Keep(EXECUTED)"


//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        let (has, _) = BigVectorTest::index_of(&account, @0x1, 10001);
        assert(has == true, 106);
    }
}
// check: gas_used
// check: 39082842
// check: "Keep(EXECUTED)"
// search 10000 need 39078951 gas

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::remove(&account, 10001);
        assert(BigVectorTest::value_of(&account, 0) == 0, 105);
        assert(BigVectorTest::value_of(&account, 10000) == 10000, 105);
    }
}
// check: gas_used
// check: 53698
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::remove(&account, 0);
    }
}

// check: gas_used
// check: 7165731


//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::remove(&account, 5000);
        assert(BigVectorTest::value_of(&account, 9998) == 10000, 107);
    }
}
// check: gas_used
// check: 3609011
// check: "EXECUTED"