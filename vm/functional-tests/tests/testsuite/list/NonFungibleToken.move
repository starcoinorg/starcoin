//! account: nftservice
//! account: alice
//! account: bob

//! new-transaction
//! sender: nftservice

// a distributed key-value map is used to store entry (token_id, address, NftToken)
// key is the token_id(:vector<u8>), stored in a sorted linked list
// value is a struct 'NftToken', contains the non fungible token
// the account address of each list node is actually the owner of the token
module NonFungibleToken {
    use 0x1::Option::{Self, Option};
    use 0x1::SortedLinkedList;
    use 0x1::Signer;
    use 0x1::Vector;

    resource struct LimitedMeta {
        init: bool,
        limited: bool,
        total: u64,
    }

    resource struct NftToken<Token> {
        token: Option<Token>
    }

    struct TransferEvent {
        from: address,
        to: address,
        token_id: vector<u8>
    }

    fun verify_hash(hash_value: vector<u8>): bool {
        Vector::length(&hash_value) == 32
    }

    public fun initialize<Token>(account: &signer, limited: bool, total: u64) {
        let sender = Signer::address_of(account);
        assert(sender == {{nftservice}}, 8000);

        assert(!limited || (limited && total > 0), 1);
        let count = total;
        if (!limited) {
            count = 0;
        };
        let limited_meta = LimitedMeta {
            init: true,
            limited: limited,
            total: count,
        };
        move_to<LimitedMeta>(account, limited_meta);
        SortedLinkedList::create_new_list<vector<u8>>(account, Vector::empty());
    }

    public fun preemptive<Token>(account: &signer, nft_service_address: address, token_id: vector<u8>, token: Token):Option<Token> {
        let (exist, location) = Self::find(copy token_id, nft_service_address);
        if (exist) return Option::some(token);

        SortedLinkedList::add_node<vector<u8>>(account, token_id, location);
        move_to<NftToken<Token>>(account, NftToken<Token>{token: Option::some(token)});
        Option::none() //preemptive success
    }

    public fun accept_token<Token>(account: &signer) {
        SortedLinkedList::empty_node<vector<u8>>(account, Vector::empty());
        move_to<NftToken<Token>>(account, NftToken<Token>{token: Option::none()});
    }

    public fun safe_transfer<Token: copyable>(account: &signer, _nft_service_address: address, token_id: vector<u8>, receiver: address) acquires NftToken {
        let sender = Signer::address_of(account);
        assert(exists<NftToken<Token>>(receiver), 100001);
        assert(Self::get_token_id(sender) == token_id, 100002);

        SortedLinkedList::move_node_to<vector<u8>>(account, receiver);
        let NftToken<Token>{ token } = move_from<NftToken<Token>>(sender);
        let receiver_wallet_mut = borrow_global_mut<NftToken<Token>>(receiver);
        receiver_wallet_mut.token = token;
    }


    public fun get_token_id(addr: address): vector<u8> {
        SortedLinkedList::get_key_of_node<vector<u8>>(addr)
    }

    // TODO: find() is expensive, will provide off-chain method
    public fun find(token_id: vector<u8>, head_address: address): (bool, address) {
        SortedLinkedList::find<vector<u8>>(token_id, head_address)
    }

    struct TestNft {}
    public fun new_test_nft(): TestNft {
        TestNft{}
    }
}


// check: EXECUTED

//! new-transaction
//! sender: nftservice
script {
use {{nftservice}}::NonFungibleToken::{Self, TestNft};
fun main(account: &signer) {
    NonFungibleToken::initialize<TestNft>(account, false, 0);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken::{Self, TestNft};
use 0x1::Hash;
fun main(account: &signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    let nft_service_address = {{nftservice}};
    let token = NonFungibleToken::new_test_nft();
    NonFungibleToken::preemptive<TestNft>(account, nft_service_address, token_id, token);
}
}

// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use {{nftservice}}::NonFungibleToken::{Self, TestNft};
fun main(account: &signer) {
    NonFungibleToken::accept_token<TestNft>(account);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken::{Self, TestNft};
use 0x1::Hash;
fun main(account: &signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    let nft_service_address = {{nftservice}};
    let receiver = {{bob}};
    NonFungibleToken::safe_transfer<TestNft>(account, nft_service_address, token_id, receiver);
}
}

// check: EXECUTED