//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# faucet --addr bob

//# publish
module alice::BigVectorTest {
    use StarcoinFramework::Vector;
    use StarcoinFramework::Signer;

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
    }

    public fun append(account: &signer, num: u64) acquires BigVector {
        let big_vec = borrow_global_mut<BigVector<u64>>(Signer::address_of(account));
        let vec = &mut big_vec.vec;
        let index = Vector::length(vec);
        let total = index + num;
        while (index < total) {
            let element = Element<u64> {
                addr: @0x1,
                value: index,
            };
            Vector::push_back(vec, element);
            index = index + 1;
        }
    }

    public fun value_of(account: &signer, index: u64): u64 acquires BigVector {
        let vec = &borrow_global<BigVector<u64>>(Signer::address_of(account)).vec;
        Vector::borrow<Element<u64>>(vec, index).value
    }

    public fun remove(account: &signer, index: u64) acquires BigVector {
        let vec = &mut borrow_global_mut<BigVector<u64>>(Signer::address_of(account)).vec;
        let Element {addr: _, value: _} = Vector::remove<Element<u64>>(vec, index);
    }

    public fun index_of(account: &signer, addr: address, value: u64): (bool, u64) acquires BigVector {
        let vec = &borrow_global<BigVector<u64>>(Signer::address_of(account)).vec;
        let element = Element<u64> {
            addr,
            value,
        };
        let (hass, index) = Vector::index_of(vec, &element);
        let Element<u64> {addr: _, value: _} = element;
        (hass, index)
    }


}

//# run --signers alice
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::init(&account);
        assert!(BigVectorTest::value_of(&account, 4999) == 4999, 101);
    }
}

//# run --signers alice
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::append(&account, 1);
        assert!(BigVectorTest::value_of(&account, 5000) == 5000, 102);
    }
}

//# run --signers alice
// appand 5000 elements "5000, 5001, ... 10000" to the vector
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::append(&account, 5000);
        assert!(BigVectorTest::value_of(&account, 10000) == 10000, 103);
    }
}

//# run --signers alice
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::append(&account, 1);
        assert!(BigVectorTest::value_of(&account, 10001) == 10001, 104);
    }
}

//# run --signers alice
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        let (hass, _) = BigVectorTest::index_of(&account, @0x1, 10001);
        assert!(hass == true, 106);
    }
}

//# run --signers alice
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::remove(&account, 10001);
        assert!(BigVectorTest::value_of(&account, 0) == 0, 105);
        assert!(BigVectorTest::value_of(&account, 10000) == 10000, 105);
    }
}

//# run --signers alice
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::remove(&account, 0);
    }
}

//# run --signers alice
script {
    use alice::BigVectorTest;
    fun main(account: signer) {
        BigVectorTest::remove(&account, 5000);
        assert!(BigVectorTest::value_of(&account, 9998) == 10000, 107);
    }
}
