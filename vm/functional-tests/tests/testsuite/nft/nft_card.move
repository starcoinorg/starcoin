//! account: creator
//! account: bob

//! sender: creator
address creator = {{creator}};
module creator::Card {
    use 0x1::NFT::{Self, NFT, MintCapability, BurnCapability};
    use 0x1::Timestamp;

    struct L1Card has store, drop{
        gene: u64,
    }
    struct L2Card has store, drop{
        first: L1Card,
        second: L1Card,
    }

    struct L1CardMintCapability has key{
        cap: MintCapability<L1Card>,
    }

    struct L2CardMintCapability has key{
        cap: MintCapability<L2Card>,
    }

    struct L1CardBurnCapability has key{
        cap: BurnCapability<L1Card>,
    }

    struct L2CardBurnCapability has key{
        cap: BurnCapability<L2Card>,
    }

    public fun init(sender: &signer){
        NFT::register<L1Card>(sender);
        let cap = NFT::remove_mint_capability<L1Card>(sender);
        move_to(sender, L1CardMintCapability{ cap});

        let cap = NFT::remove_burn_capability<L1Card>(sender);
        move_to(sender, L1CardBurnCapability{ cap});

        NFT::register<L2Card>(sender);
        let cap = NFT::remove_mint_capability<L2Card>(sender);
        move_to(sender, L2CardMintCapability{ cap});

        let cap = NFT::remove_burn_capability<L2Card>(sender);
        move_to(sender, L2CardBurnCapability{ cap});
    }

    public fun mint_l1(sender: &signer): NFT<L1Card> acquires L1CardMintCapability{
        let cap = borrow_global_mut<L1CardMintCapability>(@creator);
        //TODO set gene by a random oracle.
        let metadata = NFT::new_metadata(b"l1_card", b"ipfs:://xxxxxx", b"This is a L1Card nft.");
        NFT::mint_with_cap(sender, &mut cap.cap, metadata, L1Card{ gene: Timestamp::now_milliseconds()})
    }

    public fun mint_l2(sender: &signer, first: NFT<L1Card>, second: NFT<L1Card>): NFT<L2Card> acquires L1CardBurnCapability, L2CardMintCapability {
        let burn_cap = borrow_global_mut<L1CardBurnCapability>(@creator);
        let f = NFT::burn_with_cap(&mut burn_cap.cap, first);
        let s = NFT::burn_with_cap(&mut burn_cap.cap, second);
        let mint_cap = borrow_global_mut<L2CardMintCapability>(@creator);
        let metadata = NFT::new_metadata(b"l2_card", b"ipfs:://xxxxxx", b"This is a L2Card nft.");
        NFT::mint_with_cap(sender, &mut mint_cap.cap, metadata, L2Card{
            first:f,
            second:s,
        })
    }
}

// check: EXECUTED

//! new-transaction
//! sender: creator
address creator = {{creator}};
script {
    use creator::Card;
    fun main(sender: signer) {
        Card::init(&sender);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use 0x1::NFTGallery;
    use creator::Card::{L1Card, L2Card};
    fun main(sender: signer) {
        NFTGallery::accept<L1Card>(&sender);
        NFTGallery::accept<L2Card>(&sender);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use 0x1::NFTGallery;
    use creator::Card;
    fun main(sender: signer) {
        let first_l1 = Card::mint_l1(&sender);
        NFTGallery::deposit(&sender, first_l1);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use 0x1::NFTGallery;
    use creator::Card::{Self, L1Card};
    use 0x1::Signer;

    fun main(sender: signer) {
        let second_l1 = Card::mint_l1(&sender);
        NFTGallery::deposit(&sender, second_l1);
        assert(NFTGallery::count_of<L1Card>(Signer::address_of(&sender)) == 2, 1000);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use 0x1::NFTGallery;
    use 0x1::Option;
    use creator::Card::{Self, L1Card};
    fun main(sender: signer) {
        let first_l1 = NFTGallery::withdraw_one<L1Card>(&sender);
        let second_l1 = NFTGallery::withdraw_one<L1Card>(&sender);
        let l2_card = Card::mint_l2(&sender, Option::destroy_some(first_l1), Option::destroy_some(second_l1));
        NFTGallery::deposit(&sender, l2_card);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use creator::Card::{L1Card, L2Card};
    use 0x1::NFTGallery;
    use 0x1::Signer;

    fun main(sender: signer) {
        assert(NFTGallery::count_of<L1Card>(Signer::address_of(&sender)) == 0, 1001);
        assert(NFTGallery::count_of<L2Card>(Signer::address_of(&sender)) == 1, 1002);
    }
}

// check: EXECUTED