//# init -n dev

//# faucet --addr default

//# faucet --addr nameservice --amount 100000000

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol

//# faucet --addr david

//# faucet --addr vivian --amount 1000000

//# publish
module default::SortedLinkedList {
    use StarcoinFramework::Compare;
    use StarcoinFramework::BCS;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Vector;

    struct EntryHandle has copy, drop, store {
        //address where the Node is stored
        addr: address,
        //an index into the NodeVector stored under addr
        index: u64
    }

    struct Node<T> has key, store {
        //pointer to previous and next Node's in the sorted linked list
        prev: EntryHandle,
        next: EntryHandle,
        head: EntryHandle,
        data: T
    }

    //a vector of Node's stored under a single account
    struct NodeVector<T> has key, store {
        nodes: vector<Node<T>>
    }

    public fun entry_handle(addr: address, index: u64): EntryHandle {
        EntryHandle { addr, index }
    }

    public fun get_addr(entry: EntryHandle): address {
        entry.addr
    }

    public fun get_index(entry: EntryHandle): u64 {
        entry.index
    }

    public fun node_exists<T: copy + drop + store>(entry: EntryHandle): bool acquires NodeVector {
        if (!exists<NodeVector<T>>(entry.addr)) return false;
        let node_vector = &borrow_global<NodeVector<T>>(entry.addr).nodes;
        if (entry.index >= Vector::length<Node<T>>(node_vector)) return false;
        true
    }

    public fun get_data<T: copy + drop + store>(entry: EntryHandle): T acquires NodeVector {
        //make sure a node exists in entry
        assert!(node_exists<T>(copy entry), 1);
        let nodes = &borrow_global<NodeVector<T>>(entry.addr).nodes;
        let node = Vector::borrow<Node<T>>(nodes, entry.index);
        *&node.data
    }

    public fun get_prev_node_addr<T: copy + drop + store>(entry: EntryHandle): address acquires NodeVector {
        //make sure a node exists in entry
        assert!(node_exists<T>(copy entry), 2);
        let nodes = &borrow_global<NodeVector<T>>(entry.addr).nodes;
        let node = Vector::borrow<Node<T>>(nodes, entry.index);
        *&node.prev.addr
    }

    //checks whether this entry is the head of a list
    public fun is_head_node<T: copy + drop + store>(entry: &EntryHandle): bool acquires NodeVector {
		//check that a node exists
        assert!(node_exists<T>(*entry), 3);
        let nodes = &borrow_global<NodeVector<T>>(entry.addr).nodes;
        //find the head node
        let node = Vector::borrow<Node<T>>(nodes, entry.index);

        //check if this is the head node
        node.head.addr == entry.addr && node.head.index == entry.index
    }

    //creates a new list whose head is at txn_sender (is owned by the caller)
    public fun create_new_list<T: copy + drop + store>(account: &signer, data: T) {
        let sender = Signer::address_of(account);

        //make sure no node/list is already stored in this account
        assert!(!exists<NodeVector<T>>(sender), 3);
        let head_handle = entry_handle(sender, 0);
        let head = Self::Node<T> {
            prev: copy head_handle,
            next: copy head_handle,
            head: head_handle,
            data: data
        };

        let node_vector = Vector::singleton(head);
        move_to<NodeVector<T>>(account, NodeVector<T> { nodes: node_vector });
    }

    //adds a node that is stored in txn_sender's account and whose location in the list is right after prev_node_address
    public fun insert_node<T: copy + drop + store>(account: &signer, data: T, prev_entry: EntryHandle) acquires NodeVector {
        let sender_address = Signer::address_of(account);

        //make sure a node exists in prev_entry
        assert!(node_exists<T>(copy prev_entry), 1);
        let prev_nodes = &borrow_global<NodeVector<T>>(prev_entry.addr).nodes;

        //get a reference to prev_node and find the address and reference to next_node, head
        let prev_node = Vector::borrow(prev_nodes, prev_entry.index);
        let next_entry = *&prev_node.next;
        let next_node_vector = &borrow_global<NodeVector<T>>(next_entry.addr).nodes;
        let next_node = Vector::borrow(next_node_vector, next_entry.index);
        let head_entry = *&next_node.head;

        //see if either prev or next are the head and get their datas
        let prev_data = *&prev_node.data;
        let next_data = *&next_node.data;
        let data_bcs_bytes = BCS::to_bytes(&data);
        let cmp_with_prev = Compare::cmp_bcs_bytes(&data_bcs_bytes, &BCS::to_bytes(&prev_data));
        let cmp_with_next = Compare::cmp_bcs_bytes(&data_bcs_bytes, &BCS::to_bytes(&next_data));

        let prev_is_head = Self::is_head_node<T>(&prev_entry);
        let next_is_head = Self::is_head_node<T>(&next_entry);

        //check the order -- the list must be sorted
        assert!(prev_is_head || cmp_with_prev == 2u8, 6); // prev_is_head || data > prev_data
        assert!(next_is_head || cmp_with_next == 1u8, 7); // next_is_head || data < next_data

        //create the new node
        let node = Self::Node<T> {
            prev: copy prev_entry,
            next: copy next_entry,
            head: head_entry,
            data: data
        };

        let index = 0u64;
        if (!exists<NodeVector<T>>(sender_address)) {
            move_to<NodeVector<T>>(account, NodeVector<T> { nodes: Vector::singleton(node) });
        } else {
            let node_vector_mut = &mut borrow_global_mut<NodeVector<T>>(sender_address).nodes;
            Vector::push_back<Node<T>>(node_vector_mut, node);
            index = Vector::length<Node<T>>(node_vector_mut) - 1;
        };

        let prev_node_vector_mut = &mut borrow_global_mut<NodeVector<T>>(prev_entry.addr).nodes;
        let prev_node_mut = Vector::borrow_mut(prev_node_vector_mut, prev_entry.index);
        //fix the pointers at prev
        prev_node_mut.next.addr = sender_address;
        prev_node_mut.next.index = index;

        let next_node_vector_mut = &mut borrow_global_mut<NodeVector<T>>(next_entry.addr).nodes;
        let next_node_mut = Vector::borrow_mut(next_node_vector_mut, next_entry.index);
        //fix the pointers at next
        next_node_mut.prev.addr = sender_address;
        next_node_mut.prev.index = index;
    }

    //private function used for removing a non-head node -- does not check permissions
    fun remove_node<T: copy + drop + store>(entry: EntryHandle) acquires NodeVector {
        //check that a node exists
        assert!(node_exists<T>(copy entry), 1);
        let nodes = &borrow_global<NodeVector<T>>(entry.addr).nodes;

        //find prev and next
        let current_node = Vector::borrow(nodes, entry.index);
        let prev_entry = *&current_node.prev;
        let next_entry = *&current_node.next;

        let prev_node_vector_mut = &mut borrow_global_mut<NodeVector<T>>(prev_entry.addr).nodes;
        let prev_node_mut = Vector::borrow_mut(prev_node_vector_mut, prev_entry.index);
        //fix the pointers at prev
        prev_node_mut.next.addr = next_entry.addr;
        prev_node_mut.next.index = next_entry.index;

        let next_node_vector_mut = &mut borrow_global_mut<NodeVector<T>>(next_entry.addr).nodes;
        let next_node_mut = Vector::borrow_mut(next_node_vector_mut, next_entry.index);
        //fix the pointers at next
        next_node_mut.prev.addr = prev_entry.addr;
        next_node_mut.prev.index = prev_entry.index;

        let node_vector_mut = &mut borrow_global_mut<NodeVector<T>>(entry.addr).nodes;
        //destroy the current node
        let Node<T> { prev: _, next: _, head: _, data: _ } = Vector::remove<Node<T>>(node_vector_mut, entry.index);
    }

    public fun remove_node_by_list_owner<T: copy + drop + store>(account: &signer, entry: EntryHandle) acquires NodeVector {
        //check that a node exists
        assert!(node_exists<T>(copy entry), 1);
        //make sure it is not a head node
        assert!(!Self::is_head_node<T>(&copy entry), 10);
        //make sure the caller owns the list

        let nodes = &borrow_global<NodeVector<T>>(entry.addr).nodes;
        let current_node = Vector::borrow(nodes, entry.index);
        let list_owner = current_node.head.addr;
        assert!(list_owner == Signer::address_of(account), 11);

        //remove it
        Self::remove_node<T>(entry);
    }

    //removes the current non-head node -- fails if the passed node is the head of a list
    public fun remove_node_by_node_owner<T: copy + drop + store>(account: &signer, entry: EntryHandle) acquires NodeVector {
        //check that a node exists
        assert!(node_exists<T>(copy entry), 1);
        //make sure it is not a head node
        assert!(!Self::is_head_node<T>(&copy entry), 10);
        //make sure the caller owns the node
        assert!(entry.addr == Signer::address_of(account), 11);

        //remove it
        Self::remove_node<T>(entry);
    }

    //can only called by the list owner (head) -- removes the list if it is empty,
    //fails if it is non-empty or if no list is owned by the caller
    public fun remove_list<T: copy + drop + store>(account: &signer) acquires NodeVector {
        let sender_address = Signer::address_of(account);

        //fail if the caller does not own a list
        assert!(Self::is_head_node<T>(&Self::entry_handle(sender_address, 0)), 14);

        let node_vector = &borrow_global<NodeVector<T>>(sender_address).nodes;
        let current_node = Vector::borrow(node_vector, 0);

        //check that the list is empty
        assert!(current_node.next.addr == sender_address, 15);
        assert!(current_node.next.index == 0, 16);
        assert!(current_node.prev.addr == sender_address, 17);
        assert!(current_node.prev.index == 0, 18);

        //destroy the Node
        let NodeVector { nodes: nodes } = move_from<NodeVector<T>>(sender_address);
        let Node<T> { prev: _, next: _, head: _, data: _ } = Vector::remove<Node<T>>(&mut nodes, 0);
        Vector::destroy_empty(nodes);
    }

    public fun find_position_and_insert<T: copy + drop + store>(account: &signer, data: T, head: EntryHandle): bool acquires NodeVector {
        assert!(Self::is_head_node<T>(&copy head), 18);

        let data_bcs_bytes = BCS::to_bytes(&data);
        let nodes = &borrow_global<NodeVector<T>>(head.addr).nodes;
        let head_node = Vector::borrow<Node<T>>(nodes, head.index);
        let next_entry = *&head_node.next;
        let last_entry = *&head_node.prev;

        while (!Self::is_head_node<T>(&next_entry)) {
            let next_nodes = &borrow_global<NodeVector<T>>(next_entry.addr).nodes;
            let next_node = Vector::borrow<Node<T>>(next_nodes, next_entry.index);

            let next_node_data = *&next_node.data;
            let next_data_bcs_bytes = BCS::to_bytes(&next_node_data);
            let cmp = Compare::cmp_bcs_bytes(&next_data_bcs_bytes, &data_bcs_bytes);

            if (cmp == 0u8) { // next_data == data
                return false  // data already exist
            } else if (cmp == 1u8) { // next_data < data, continue
                next_entry = *&next_node.next;
            } else { // next_data > data, nothing found
                let prev_entry = *&next_node.prev;
                insert_node(account, data, prev_entry);
                return true
            }
        };
        // list is empty, insert after head
        insert_node(account, data, last_entry);
        true
    }

}


//# run --signers alice
//creating a new list _@alice
script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        SortedLinkedList::create_new_list<u64>(&account, 0);
    }
}
// check: EXECUTED

//# run --signers alice
//attempting to create another list with the same head

script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        SortedLinkedList::create_new_list<u64>(&account, 0);
    }
}
// check: ABORTED
// check: 1

//# run --signers bob
//adding a new element to Alice's list _@alice -> 10@bob

script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        // let prev_entry = SortedLinkedList::entry_handle(@alice, 0);
    // SortedLinkedList::insert_node<u64>(&account, 10, prev_entry);
    let head_entry = SortedLinkedList::entry_handle(@alice, 0);
        SortedLinkedList::find_position_and_insert(&account, 10, head_entry);
    }
}
// check: EXECUTED

//# run --signers carol
//adding a new element to Alice's list _@alice -> 10@bob -> 12@carol

script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        // let prev_entry = SortedLinkedList::entry_handle(@bob, 0);
    // SortedLinkedList::insert_node<u64>(&account, 12, prev_entry);
    let head_entry = SortedLinkedList::entry_handle(@alice, 0);
        SortedLinkedList::find_position_and_insert(&account, 12, head_entry);
    }
}
// check: EXECUTED

//# run --signers carol
//adding a new element to Alice's list _@alice -> 10@bob -> 11@carol -> 12@carol

script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        let head_entry = SortedLinkedList::entry_handle(@alice, 0);
        SortedLinkedList::find_position_and_insert(&account, 11, head_entry);
    }
}
// check: EXECUTED

//# run --signers alice
//check the list _@alice -> 10@bob -> 11@carol -> 12@carol

script {
    use default::SortedLinkedList;
    fun main() {
        let entry0 = SortedLinkedList::entry_handle(@alice, 0);
        assert!(SortedLinkedList::get_data(copy entry0) == 0, 29);
        assert!(SortedLinkedList::get_prev_node_addr<u64>(entry0) == @carol, 30);
        let entry1 = SortedLinkedList::entry_handle(@bob, 0);
        assert!(SortedLinkedList::get_data(copy entry1) == 10, 31);
        assert!(SortedLinkedList::get_prev_node_addr<u64>(entry1) == @alice, 34);
        let entry2 = SortedLinkedList::entry_handle(@carol, 1);
        assert!(SortedLinkedList::get_data(copy entry2) == 11, 32);
        assert!(SortedLinkedList::get_prev_node_addr<u64>(entry2) == @bob, 35);
        let entry3 = SortedLinkedList::entry_handle(@carol, 0);
        assert!(SortedLinkedList::get_data(copy entry3) == 12, 33);
        assert!(SortedLinkedList::get_prev_node_addr<u64>(entry3) == @carol, 36);
    }
}
// check: EXECUTED

//# run --signers alice
//Alice removes Bob's node _@alice -> 11@carol -> 12@carol

script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        let entry = SortedLinkedList::entry_handle(@bob, 0);
        SortedLinkedList::remove_node_by_list_owner<u64>(&account, entry);
    }
}
// check: EXECUTED

//# run --signers carol
//David removes his node _@alice -> 12@carol

script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        let entry = SortedLinkedList::entry_handle(@carol, 1);
        SortedLinkedList::remove_node_by_node_owner<u64>(&account, entry);
    }
}
// check: EXECUTED

//# run --signers alice
//Alice empties her list and removes it
script {
    use default::SortedLinkedList;
    fun main(account: signer) {
        let entry = SortedLinkedList::entry_handle(@carol, 0);
        SortedLinkedList::remove_node_by_list_owner<u64>(&account, entry);
        SortedLinkedList::remove_list<u64>(&account);
    }
}
// check: EXECUTED



//# publish

// a distributed key-value map is used to store name entry (name, address, expiration_date)
// key is the name(:vector<u8>), stored in a sorted linked list
// value is a struct 'Expiration', contains the expiration date of the name
// the account address of each list node is actually the address bound to the key(name)

module nameservice::NameService {
    use default::SortedLinkedList::{Self, EntryHandle};
    use StarcoinFramework::Block;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Vector;

    //TODO use constants when Move support constants, '5' is used for example
    const EXPIRE_AFTER: u64 = 5;

    struct Expiration has key, store {
        expire_on_block_number: vector<u64>
    }

    public fun entry_handle(addr: address, index: u64): EntryHandle {
        SortedLinkedList::entry_handle(addr, index)
    }

    public fun initialize(account: &signer) {
        let sender = Signer::address_of(account);
        assert!(sender == @nameservice, 8000);

        SortedLinkedList::create_new_list<vector<u8>>(account, Vector::empty());
        move_to<Expiration>(account, Expiration { expire_on_block_number: Vector::singleton(0u64)});
    }

    fun add_expirtation(account: &signer) acquires Expiration {
        let sender = Signer::address_of(account);
        let current_block = Block::get_current_block_number();
        if (!exists<Expiration>(sender)) {
            move_to<Expiration>(account, Expiration {expire_on_block_number: Vector::singleton(current_block + EXPIRE_AFTER)});
        } else {
            let expire_vector_mut = &mut borrow_global_mut<Expiration>(sender).expire_on_block_number;
            Vector::push_back<u64>(expire_vector_mut, current_block + EXPIRE_AFTER);
        };
    }

    public fun add_name(account: &signer, name: vector<u8>, prev_entry: EntryHandle) acquires Expiration {
        SortedLinkedList::insert_node(account, name, prev_entry);
        Self::add_expirtation(account);
    }

    public fun get_name_for(entry: EntryHandle): vector<u8> {
        SortedLinkedList::get_data<vector<u8>>(entry)
    }

    fun remove_expiration(entry: EntryHandle) acquires Expiration {
        let account_address = SortedLinkedList::get_addr(copy entry);
        let index = SortedLinkedList::get_index(entry);
        let expire_vector_mut = &mut borrow_global_mut<Expiration>(account_address).expire_on_block_number;
        Vector::remove<u64>(expire_vector_mut, index);
        if (Vector::is_empty<u64>(expire_vector_mut)) {
            let Expiration { expire_on_block_number } = move_from<Expiration>(account_address);
            Vector::destroy_empty(expire_on_block_number);
        }
    }
    public fun remove_entry_by_entry_owner(account: &signer, entry: EntryHandle) acquires Expiration {
        SortedLinkedList::remove_node_by_node_owner<vector<u8>>(account, copy entry);
        Self::remove_expiration(entry);
    }

    public fun remove_entry_by_service_owner(account: &signer, entry: EntryHandle) acquires Expiration {
        SortedLinkedList::remove_node_by_list_owner<vector<u8>>(account, copy entry);
        Self::remove_expiration(entry);
    }

    public fun find_position_and_insert(account: &signer, name: vector<u8>, head: EntryHandle): bool acquires Expiration {
        if (SortedLinkedList::find_position_and_insert<vector<u8>>(account, name, head)) {
            Self::add_expirtation(account);
            return true
        } else {
            return false
        }
    }

    public fun is_head_entry(entry: EntryHandle): bool {
		SortedLinkedList::is_head_node<vector<u8>>(&entry)
    }

    public fun expire_on_block_number(entry: EntryHandle): u64 acquires Expiration {
        let addr = SortedLinkedList::get_addr(copy entry);
        let index = SortedLinkedList::get_index(entry);
        let expire_vector = *&borrow_global<Expiration>(addr).expire_on_block_number;
        *Vector::borrow<u64>(&expire_vector, index)
    }

    public fun is_expired(entry: EntryHandle): bool acquires Expiration {
        let current_block_number = Block::get_current_block_number();
        current_block_number > expire_on_block_number(entry)
    }

}

//# run --signers nameservice
//initialize the nameservice list
script {
use nameservice::NameService;
fun main(account: signer) {
    NameService::initialize(&account);
}
}
// check: EXECUTED

//# run --signers alice
//adding a new name to NameService's list _@nameservice -> b"alice"@alice
script {
use nameservice::NameService;
fun main(account: signer) {
    let head = NameService::entry_handle(@nameservice, 0);
    NameService::find_position_and_insert(&account, b"alice", head);
}
}
// check: EXECUTED

//# run --signers bob
//adding a new name to NameService's list _@nameservice -> b"bob"@bob -> b"alice"@alice
script {
use nameservice::NameService;
fun main(account: signer) {
    let head = NameService::entry_handle(@nameservice, 0);
    NameService::find_position_and_insert(&account, b"bob", head);
}
}
// check: EXECUTED

//# run --signers carol
//adding a new name to NameService's list _@nameservice -> b"bob"@bob -> b"alice"@alice -> b"carol"@carol
script {
use nameservice::NameService;
fun main(account: signer) {
    let head = NameService::entry_handle(@nameservice, 0);
    NameService::find_position_and_insert(&account, b"carol", head);
}
}
// check: EXECUTED

//# run --signers david
//ensure the entry under alice holds the name b"alice"
script {
use nameservice::NameService;
fun main() {
    let entry = NameService::entry_handle(@alice, 0);
    let name = NameService::get_name_for(entry);
    assert!(name == b"alice", 26);
}
}
// check: EXECUTED

//# run --signers carol
//removes her entry _@nameservice -> b"bob"@bob -> b"alice"@alice
script {
use nameservice::NameService;
fun main(account: signer) {
    let entry = NameService::entry_handle(@carol, 0);
    NameService::remove_entry_by_entry_owner(&account, entry);
}
}
// check: EXECUTED

//# run --signers nameservice
//removes bob's entry _@nameservice -> b"alice"@alice
script {
use nameservice::NameService;
fun main(account: signer) {
    let entry = NameService::entry_handle(@bob, 0);
    assert!(NameService::is_expired(copy entry), 27);
    NameService::remove_entry_by_service_owner(&account, entry);
}
}
// check: ABORTED


//# block --author vivian --timestamp 1001000

//# block --author vivian --timestamp 1002000

//# block --author vivian --timestamp 1003000

//# block --author vivian --timestamp 1004000

//# block --author vivian --timestamp 1005000

//# block --author vivian --timestamp 1006000

//# run --signers nameservice
//removes her entry _@nameservice -> b"alice"@alice
script {
use nameservice::NameService;
fun main(account: signer) {
    let entry = NameService::entry_handle(@bob, 0);
    assert!(NameService::is_expired(copy entry), 27);
    NameService::remove_entry_by_service_owner(&account, entry);
}
}
// check: EXECUTED