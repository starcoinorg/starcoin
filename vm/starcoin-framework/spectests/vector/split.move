//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use StarcoinFramework::Vector;
    fun split() {

        let vec = Vector::empty<u64>();

        Vector::push_back<u64>(&mut vec, 0);
        Vector::push_back<u64>(&mut vec, 1);
        Vector::push_back<u64>(&mut vec, 2);
        Vector::push_back<u64>(&mut vec, 3);

        //split [0,1,2,3] into [0,1],[2,3]
        let new = Vector::split<u64>(&mut vec, 2);

        let member = *Vector::borrow<vector<u64>>(&mut new, 0);
        assert!(*Vector::borrow<u64>(&member, 0) == 0, 1001);
        assert!(*Vector::borrow<u64>(&member, 1) == 1, 1002);

        let member = *Vector::borrow<vector<u64>>(&mut new, 1);
        assert!(*Vector::borrow<u64>(&member, 0) == 2, 1003);
        assert!(*Vector::borrow<u64>(&member, 1) == 3, 1004);
    }
}

//# run --signers alice
script {
    use StarcoinFramework::Vector;
    fun split() {

        let vec = Vector::empty<u64>();

        Vector::push_back<u64>(&mut vec, 0);
        Vector::push_back<u64>(&mut vec, 1);
        Vector::push_back<u64>(&mut vec, 2);
        Vector::push_back<u64>(&mut vec, 3);
        Vector::push_back<u64>(&mut vec, 4);

        //split [0,1,2,3,4] into [0,1],[2,3],[4]
        let new = Vector::split<u64>(&mut vec, 2);

        let member = *Vector::borrow<vector<u64>>(&mut new, 0);
        assert!(*Vector::borrow<u64>(&member, 0) == 0, 1001);
        assert!(*Vector::borrow<u64>(&member, 1) == 1, 1002);

        let member = *Vector::borrow<vector<u64>>(&mut new, 1);
        assert!(*Vector::borrow<u64>(&member, 0) == 2, 1003);
        assert!(*Vector::borrow<u64>(&member, 1) == 3, 1004);

        let member = *Vector::borrow<vector<u64>>(&mut new, 2);
        assert!(*Vector::borrow<u64>(&member, 0) == 4, 1005);
    }
}
