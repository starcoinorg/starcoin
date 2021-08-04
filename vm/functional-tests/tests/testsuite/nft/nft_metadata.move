//! account: creator
//! account: bob

//! sender: creator
address creator = {{creator}};
module creator::Card {
    use 0x1::NFT::{Self, NFT, MintCapability, BurnCapability, UpdateCapability};

    struct Card has copy, store, drop{
        level: u64
    }
  
    struct CardMintCapability has key{
        cap: MintCapability<Card>,
    }

    struct CardBurnCapability has key{
        cap: BurnCapability<Card>,
    }

    struct CardUpdateCapability has key{
        cap: UpdateCapability<Card>,
    }

    public fun get_level(card: &Card): u64 {
        card.level
    }

    public fun init(sender: &signer){
        NFT::register<Card>(sender);
        let cap = NFT::remove_mint_capability<Card>(sender);
        move_to(sender, CardMintCapability{ cap});

        let cap = NFT::remove_burn_capability<Card>(sender);
        move_to(sender, CardBurnCapability{ cap});

        let cap = NFT::remove_update_capability<Card>(sender);
        move_to(sender, CardUpdateCapability{ cap});
    }

    public fun mint(sender: &signer): NFT<Card> acquires CardMintCapability{
        let cap = borrow_global_mut<CardMintCapability>(@creator);
        let metadata = NFT::new_meta_with_image(b"card", b"ipfs:://xxxxxx", b"This is a Card nft.");
        NFT::mint_with_cap(sender, &mut cap.cap, metadata, Card{ level: 1})
    }

    /// upgrade the first card by burn the second card.
    public fun upgrade_card(first: &mut NFT<Card>, second: NFT<Card>) acquires CardBurnCapability, CardUpdateCapability {
        let burn_cap = borrow_global_mut<CardBurnCapability>(@creator);
        let first_card_meta = NFT::get_type_meta(first);
        let second_card_meta = NFT::burn_with_cap(&mut burn_cap.cap, second);

        let update_cap = borrow_global_mut<CardUpdateCapability>(@creator);
        let metadata = *NFT::get_base_meta(first);
        NFT::update_meta_with_cap(&mut update_cap.cap, first, metadata, Card{
            level: first_card_meta.level + second_card_meta.level
        });
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
    use creator::Card::{Card};
    fun main(sender: signer) {
        NFTGallery::accept<Card>(&sender);
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
        let first = Card::mint(&sender);
        NFTGallery::deposit(&sender, first);
        let second = Card::mint(&sender);
        NFTGallery::deposit(&sender, second);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use 0x1::NFTGallery;
    use 0x1::Option;
    use creator::Card::{Self, Card};
    fun main(sender: signer) {
        let first = Option::destroy_some(NFTGallery::withdraw_one<Card>(&sender));
        let second = NFTGallery::withdraw_one<Card>(&sender);
        Card::upgrade_card(&mut first, Option::destroy_some(second));
        NFTGallery::deposit(&sender, first);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use creator::Card::{Self, Card};
    use 0x1::NFTGallery;
    use 0x1::Signer;
    use 0x1::Option;
    use 0x1::NFT;

    fun main(sender: signer) {
        assert(NFTGallery::count_of<Card>(Signer::address_of(&sender)) == 1, 1001);
        let card = Option::destroy_some(NFTGallery::withdraw_one<Card>(&sender));
        let card_meta = NFT::get_type_meta(&card);
        let level = Card::get_level(card_meta);
        assert(level == 2, 1002);
        NFTGallery::deposit(&sender, card);
    }
}

// check: EXECUTED