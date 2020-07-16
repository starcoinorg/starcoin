//! account: alice, 100000000
//! account: bob
//! account: carol

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
    // let prev_entry = SortedLinkedList::entry_handle({{alice}}, 0);
    // SortedLinkedList::insert_node<u64>(account, 10, prev_entry);
    let head_entry = SortedLinkedList::entry_handle({{alice}}, 0);
    SortedLinkedList::find_position_and_insert(account, 10, head_entry);
}
}
// check: EXECUTED

//! new-transaction
//! sender: carol
//adding a new element to Alice's list _@alice -> 10@bob -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    // let prev_entry = SortedLinkedList::entry_handle({{bob}}, 0);
    // SortedLinkedList::insert_node<u64>(account, 12, prev_entry);
    let head_entry = SortedLinkedList::entry_handle({{alice}}, 0);
    SortedLinkedList::find_position_and_insert(account, 12, head_entry);
}
}
// check: EXECUTED

//! new-transaction
//! sender: carol
//adding a new element to Alice's list _@alice -> 10@bob -> 11@carol -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    let head_entry = SortedLinkedList::entry_handle({{alice}}, 0);
    SortedLinkedList::find_position_and_insert(account, 11, head_entry);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//check the list _@alice -> 10@bob -> 11@carol -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main() {
    let entry0 = SortedLinkedList::entry_handle({{alice}}, 0);
    assert(SortedLinkedList::get_data(copy entry0) == 0, 29);
    assert(SortedLinkedList::get_prev_node_addr<u64>(entry0) == {{carol}}, 30);
    let entry1 = SortedLinkedList::entry_handle({{bob}}, 0);
    assert(SortedLinkedList::get_data(copy entry1) == 10, 31);
    assert(SortedLinkedList::get_prev_node_addr<u64>(entry1) == {{alice}}, 34);
    let entry2 = SortedLinkedList::entry_handle({{carol}}, 1);
    assert(SortedLinkedList::get_data(copy entry2) == 11, 32);
    assert(SortedLinkedList::get_prev_node_addr<u64>(entry2) == {{bob}}, 35);
    let entry3 = SortedLinkedList::entry_handle({{carol}}, 0);
    assert(SortedLinkedList::get_data(copy entry3) == 12, 33);
    assert(SortedLinkedList::get_prev_node_addr<u64>(entry3) == {{carol}}, 36);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//Alice removes Bob's node _@alice -> 11@carol -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    let entry = SortedLinkedList::entry_handle({{bob}}, 0);
    SortedLinkedList::remove_node_by_list_owner<u64>(account, entry);
}
}
// check: EXECUTED

//! new-transaction
//! sender: carol
//David removes his node _@alice -> 12@carol
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    let entry = SortedLinkedList::entry_handle({{carol}}, 1);
    SortedLinkedList::remove_node_by_node_owner<u64>(account, entry);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//Alice empties her list and removes it
script {
use 0x1::SortedLinkedList;
fun main(account: &signer) {
    let entry = SortedLinkedList::entry_handle({{carol}}, 0);
    SortedLinkedList::remove_node_by_list_owner<u64>(account, entry);
    SortedLinkedList::remove_list<u64>(account);
}
}
// check: EXECUTED