// Test NFT Module
//! account: alice
//! account: bob

//! sender: alice
module NonFungibleToken {
    use 0x1::Vector;
    use 0x1::Signer;
    use 0x1::Option::{Self, Option};
    use 0x1::Account;

    struct HashEntry {
        data: vector<u8>,
    }

    struct LimitedMeta {
        init: bool,
        limited: bool,
        total: u64,
    }

    fun verify_hash(hash_entry: &HashEntry): bool {
        let len = Vector::length(&hash_entry.data);
        if (len == 32) {
            true
        } else {
            false
        }
    }

    struct TransferEvent {
        from: address,
        to: address,
        token_id: HashEntry
    }

    resource struct NFT<Token> {
        address_vec: vector<address>,
        token_id_vec: vector<HashEntry>,
        token_vec: vector<Token>,
        config: LimitedMeta,
    }

    public fun initialize<Token>(create_account: &signer, limited: bool, total: u64) {
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
        move_to<NFT<Token>>(
            create_account,
            NFT<Token> {
                address_vec: Vector::empty(),
                token_id_vec: Vector::empty(),
                token_vec: Vector::empty(),
                config: limited_meta,
            }
        );
    }

    public fun preemptive<Token>(create_nft_address: address, preemptive_account: &signer, token_id: vector<u8>, token: Token):Option<Token> acquires NFT {
        let token_key = HashEntry {
            data: token_id
        };

        if (exist_inner<Token>(create_nft_address, &token_key)) return Option::some(token);

        let nft = borrow_global_mut<NFT<Token>>(create_nft_address);
        let current_token_id_len = Vector::length(&nft.token_id_vec);
        let _current_address_len = Vector::length(&nft.address_vec);
        let _current_token_len = Vector::length(&nft.token_vec);
        if (nft.config.limited && current_token_id_len >= nft.config.total) return Option::some(token);

        if (Vector::contains(&nft.token_id_vec, &token_key)) return Option::some(token);

        let preemptive_address = Signer::address_of(preemptive_account);
        Vector::push_back(&mut nft.token_id_vec, token_key);
        Vector::push_back(&mut nft.address_vec, preemptive_address);
        Vector::push_back(&mut nft.token_vec, token);

        Option::none()
    }

    public fun safe_transfer<Token>(create_nft_address: address, owner_account: &signer, token_id: vector<u8>, transfer_address: address) acquires NFT {
        let transfer_address_exist = Account::exists_at(transfer_address);
        assert(transfer_address_exist, 100001);

        let token_key = HashEntry {
            data: token_id
        };
        assert(exist_inner<Token>(create_nft_address, &token_key), 100002);

        let owner_address = Signer::address_of(owner_account);
        let nft = borrow_global_mut<NFT<Token>>(create_nft_address);

        let (_, index) = Vector::index_of(&nft.token_id_vec, &token_key);
        let old_address = Vector::borrow(&nft.address_vec, index);
        assert(old_address == &owner_address, 100003);
        let len = Vector::length(&nft.address_vec);
        Vector::push_back(&mut nft.address_vec, transfer_address);
        Vector::swap(&mut nft.address_vec, index, len);
        Vector::remove(&mut nft.address_vec, len);
    }

    public fun exist_token<Token>(create_nft_address: address, token_id: vector<u8>):bool acquires NFT {
        let token_key = HashEntry {
            data: token_id
        };
        exist_inner<Token>(create_nft_address, &token_key)
    }

    fun exist_inner<Token>(create_nft_address: address, token_key: &HashEntry):bool acquires NFT {
        assert(verify_hash(token_key), 10000);
        let nft = borrow_global<NFT<Token>>(create_nft_address);

        Vector::contains(&nft.token_id_vec, token_key)
    }

    struct TestNft {}

    public fun new_test_nft(): TestNft {
        TestNft{}
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::NonFungibleToken::{Self, TestNft};
fun main(account: &signer) {
    NonFungibleToken::initialize<TestNft>(account, false, 0);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::NonFungibleToken::{Self, TestNft};
use 0x1::Hash;
fun main(account: &signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    let create_nft_address = {{alice}};
    let token = NonFungibleToken::new_test_nft();
    NonFungibleToken::preemptive<TestNft>(create_nft_address, account, token_id, token);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::NonFungibleToken::{Self, TestNft};
use 0x1::Hash;
fun main(account: &signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    let create_nft_address = {{alice}};
    let transfer_address = {{bob}};
    NonFungibleToken::safe_transfer<TestNft>(create_nft_address, account, token_id, transfer_address);
}
}

// check: EXECUTED
