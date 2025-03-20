//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
use StarcoinFramework::Vector;
fun main() {
    let vec = Vector::empty<u64>();

    Vector::push_back<u64>(&mut vec, 0);
    Vector::push_back<u64>(&mut vec, 1);
    Vector::push_back<u64>(&mut vec, 2);
    Vector::push_back<u64>(&mut vec, 3);
    Vector::push_back<u64>(&mut vec, 4);

    Vector::reverse<u64>(&mut vec);

    assert!(*Vector::borrow<u64>(&vec, 0) == 4, 1001);
    assert!(*Vector::borrow<u64>(&vec, 1) == 3, 1002);
    assert!(*Vector::borrow<u64>(&vec, 2) == 2, 1003);
    assert!(*Vector::borrow<u64>(&vec, 3) == 1, 1004);
    assert!(*Vector::borrow<u64>(&vec, 4) == 0, 1005);
}
}

//# run --signers alice
script {
    use StarcoinFramework::Vector;
    fun main() {
        let vec = Vector::empty<u64>();
        Vector::reverse<u64>(&mut vec);
    }
}
