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

    assert!(Vector::contains<u64>(&vec, &0) == true, 1001);
    assert!(Vector::contains<u64>(&vec, &1) == true, 1002);
    assert!(Vector::contains<u64>(&vec, &5) == false, 1003);
}
}
