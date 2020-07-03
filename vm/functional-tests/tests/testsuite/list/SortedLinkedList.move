//! account: alice, 100000000
//! account: bob
//! account: carol
//! account: david

//! new-transaction
//! sender: alice
//creating a new list _@alice
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::create_new_list<u64>(account, 0);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//attempting to create another list with the same head
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::create_new_list<u64>(account, 0);
}
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: bob
//adding a new element to Alice's list _@alice -> 10@bob
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::add_node<u64>(account, 10, {{alice}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: carol
//adding a new element to Alice's list _@alice -> 10@bob -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::add_node<u64>(account, 12, {{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: carol
//get value of Bob's node
script {
use 0x1::SortedLinkedList;
fun main() {
    let value = SortedLinkedList::get_key_of_node<u64>({{bob}});
    assert(value == 10, 21);
}
}
// check: EXECUTED

//! new-transaction
//! sender: david
//adding a new element to Alice's list _@alice -> 10@bob -> 11@david -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::add_node<u64>(account, 11, {{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//Alice removes Bob's node _@alice -> 11@david -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::remove_node_by_list_owner<u64>(account, {{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: david
//David removes his node _@alice -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::remove_node_by_node_owner<u64>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//Alice empties her list and removes it
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    SortedLinkedList::remove_node_by_list_owner<u64>(account, {{carol}});
    SortedLinkedList::remove_list<u64>(account);
}
}
// check: EXECUTED