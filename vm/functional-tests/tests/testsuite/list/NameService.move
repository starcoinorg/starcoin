//! account: nameservice, 100000000
//! account: alice
//! account: bob
//! account: carol
//! account: david
//! account: vivian, 1000000, 0, validator

//! new-transaction
//! sender: nameservice

// a distributed key-value map is used to store name entry (name, address, expiration_date)
// key is the name(:vector<u8>), stored in a sorted linked list
// value is a struct 'Expiration', contains the expiration date of the name
// the account address of each list node is actually the address bound to the key(name)
module NameService {
    use 0x1::SortedLinkedList;
    use 0x1::Block;
    use 0x1::Signer;
    use 0x1::Vector;

    //TODO use constants when Move support constants, '5' is used for example
    public fun EXPIRE_AFTER() : u64{5}

    resource struct Expiration {
        expire_on_block_height: u64
    }

    public fun initialize(account: &signer) {
        let sender = Signer::address_of(account);
        assert(sender == {{nameservice}}, 8000);

        SortedLinkedList::create_new_list<vector<u8>>(account, Vector::empty());
        move_to<Expiration>(account, Expiration {expire_on_block_height: 0});
    }

    public fun add_name(account: &signer, name: vector<u8>, prev_entry_address: address) {
        let current_block = Block::get_current_block_height();
        SortedLinkedList::add_node(account, name, prev_entry_address);
        move_to<Expiration>(account, Expiration {expire_on_block_height: current_block + EXPIRE_AFTER()});
    }

    public fun get_name_for(addr: address): vector<u8> {
        SortedLinkedList::get_key_of_node<vector<u8>>(addr)
    }

    public fun remove_entry_by_entry_owner(account: &signer) acquires Expiration {
        SortedLinkedList::remove_node_by_node_owner<vector<u8>>(account);
        let Expiration { expire_on_block_height: _ } = move_from<Expiration>(Signer::address_of(account));
    }

    public fun remove_entry_by_service_owner(account: &signer, entry_address: address) acquires Expiration {
        SortedLinkedList::remove_node_by_list_owner<vector<u8>>(account, entry_address);
        let Expiration { expire_on_block_height: _ } = move_from<Expiration>(entry_address);
    }

    // TODO: find() is expensive, will provide off-chain method
    public fun find(name: vector<u8>, head_address: address): (bool, address) {
        SortedLinkedList::find<vector<u8>>(name, head_address)
    }

    public fun is_head_entry(entry_address: address): bool {
		SortedLinkedList::is_head_node<vector<u8>>(entry_address)
    }

    public fun expire_on_block_height(entry_address: address): u64 acquires Expiration {
        let entry = borrow_global<Expiration>(entry_address);
        entry.expire_on_block_height
    }

    public fun is_expired(entry_address: address): bool acquires Expiration {
        let entry = borrow_global<Expiration>(entry_address);
        let current_block_height = Block::get_current_block_height();
        current_block_height > entry.expire_on_block_height
    }
 
}

//! new-transaction
//! sender: nameservice
//initialize the nameservice list
script {
use {{nameservice}}::NameService;
fun main(account: &signer) {
    NameService::initialize(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//adding a new name to NameService's list _@nameservice -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: &signer) {
    let (exist, location) = NameService::find(b"alice", {{nameservice}});
    assert(exist == false, 20);
    assert(location == {{nameservice}}, 21);
    NameService::add_name(account, b"alice", location);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//adding a new name to NameService's list _@nameservice -> b"bob"@bob -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: &signer) {
    let (exist, location) = NameService::find(b"bob", {{nameservice}});
    assert(exist == false, 22);
    assert(location == {{nameservice}}, 23);
    NameService::add_name(account, b"bob", location);
}
}
// check: EXECUTED

//! new-transaction
//! sender: carol
//adding a new name to NameService's list _@nameservice -> b"bob"@bob -> b"alice"@alice -> b"carol"@carol
script {
use {{nameservice}}::NameService;
fun main(account: &signer) {
    let (exist, location) = NameService::find(b"carol", {{nameservice}});
    assert(exist == false, 24);
    assert(location == {{alice}}, 25);
    NameService::add_name(account, b"carol", location);
}
}
// check: EXECUTED

//! new-transaction
//! sender: david
// look up the address bound to b"alice"
script {
use {{nameservice}}::NameService;
fun main() {
    let (exist, address) = NameService::find(b"alice", {{nameservice}});
    assert(exist, 28);
    assert(address == {{alice}}, 29);
}
}
// check: EXECUTED

//! new-transaction
//! sender: david
//ensure the entry under {{alice}} holds the name b"alice"
script {
use {{nameservice}}::NameService;
fun main() {
    let name = NameService::get_name_for({{alice}});
    assert(name == b"alice", 26);
}
}
// check: EXECUTED

//! new-transaction
//! sender: carol
//removes her entry _@nameservice -> b"bob"@bob -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: &signer) {
    NameService::remove_entry_by_entry_owner(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: nameservice
//removes her entry _@nameservice -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: &signer) {
    assert(NameService::is_expired({{bob}}), 27);
    NameService::remove_entry_by_service_owner(account, {{bob}});
}
}
// check: ABORTED


//! block-prologue
//! proposer: vivian
//! block-time: 1

//! block-prologue
//! proposer: vivian
//! block-time: 1

//! block-prologue
//! proposer: vivian
//! block-time: 1

//! block-prologue
//! proposer: vivian
//! block-time: 1

//! block-prologue
//! proposer: vivian
//! block-time: 1

//! block-prologue
//! proposer: vivian
//! block-time: 1

//! new-transaction
//! sender: nameservice
//removes her entry _@nameservice -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: &signer) {
    assert(NameService::is_expired({{bob}}), 27);
    NameService::remove_entry_by_service_owner(account, {{bob}});
}
}
// check: EXECUTED