//# init -n dev

//# faucet --addr creator

//# faucet --addr bob

//# publish
module creator::Card {
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::NFT::{Self, NFT, Metadata, MintCapability, BurnCapability, UpdateCapability};
    use StarcoinFramework::Signer;

    struct Card has copy, store, drop{
        upgrade_time: u64,
    }

    struct CardBody has store{
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

    public fun get_level(card_body: &CardBody): u64 {
        card_body.level
    }

    public fun get_upgrade_time(card: &Card): u64 {
        card.upgrade_time
    }

    public fun init(sender: &signer){
        assert!(Signer::address_of(sender) == @creator, 1000);
        NFT::register_v2<Card>(sender, NFT::empty_meta());
        let cap = NFT::remove_mint_capability<Card>(sender);
        move_to(sender, CardMintCapability{ cap});

        let cap = NFT::remove_burn_capability<Card>(sender);
        move_to(sender, CardBurnCapability{ cap});

        let cap = NFT::remove_update_capability<Card>(sender);
        move_to(sender, CardUpdateCapability{ cap});
    }

    public fun mint(_sender: &signer): NFT<Card, CardBody> acquires CardMintCapability{
        let cap = borrow_global_mut<CardMintCapability>(@creator);
        let metadata = NFT::new_meta_with_image(b"card", b"ipfs:://xxxxxx", b"This is a Card nft.");
        NFT::mint_with_cap_v2<Card, CardBody>(@creator, &mut cap.cap, metadata, Card{ upgrade_time: Timestamp::now_milliseconds()}, CardBody{ level: 1})
    }

    /// upgrade the first card by burn the second card.
    public fun upgrade_card(first: &mut NFT<Card, CardBody>, second: NFT<Card, CardBody>) acquires CardBurnCapability, CardUpdateCapability {
        let burn_cap = borrow_global_mut<CardBurnCapability>(@creator);

        let first_card_level = {
            let first_body = NFT::borrow_body(first);
            Self::get_level(first_body)
        };
        let second_body = NFT::burn_with_cap(&mut burn_cap.cap, second);
        let CardBody{ level:second_card_level } = second_body;

        let update_cap = borrow_global_mut<CardUpdateCapability>(@creator);
        let metadata = *NFT::get_base_meta(first);
        let level = first_card_level + second_card_level;

        NFT::update_meta_with_cap(&mut update_cap.cap, first, metadata, Card{
                upgrade_time: Timestamp::now_milliseconds(),
        });
        let body = NFT::borrow_body_mut_with_cap(&mut update_cap.cap, first);
        body.level = level;
    }

    public fun update_type_info_meta(sender: &signer, meta: Metadata) acquires CardUpdateCapability{
        assert!(Signer::address_of(sender) == @creator, 1000);
        let update_cap = borrow_global_mut<CardUpdateCapability>(@creator);
        NFT::update_nft_type_info_meta_with_cap(&mut update_cap.cap, meta);
    }
}

// check: EXECUTED

//# run --signers creator
script {
    use creator::Card;
    fun main(sender: signer) {
        Card::init(&sender);
    }
}

// check: EXECUTED

//# run --signers bob
script {
    use StarcoinFramework::NFTGallery;
    use creator::Card;
    fun main(sender: signer) {
        let first = Card::mint(&sender);
        NFTGallery::deposit(&sender, first);
        let second = Card::mint(&sender);
        NFTGallery::deposit(&sender, second);
    }
}

// check: EXECUTED


//# block --author 0x1

//# run --signers bob
script {
    use StarcoinFramework::NFTGallery;
    use creator::Card::{Self, Card, CardBody};
    fun main(sender: signer) {
        let first = NFTGallery::withdraw_one<Card, CardBody>(&sender);
        let second = NFTGallery::withdraw_one<Card, CardBody>(&sender);
        Card::upgrade_card(&mut first, second);
        NFTGallery::deposit(&sender, first);
    }
}

// check: EXECUTED

//# run --signers bob
script {
    use creator::Card::{Self, Card, CardBody};
    use StarcoinFramework::NFTGallery;
    use StarcoinFramework::Signer;
    use StarcoinFramework::NFT;

    fun main(sender: signer) {
        assert!(NFTGallery::count_of<Card, CardBody>(Signer::address_of(&sender)) == 1, 1001);
        let card = NFTGallery::withdraw_one<Card, CardBody>(&sender);
        let card_meta = NFT::get_type_meta(&card);
        let upgrade_time = Card::get_upgrade_time(card_meta);
        assert!(upgrade_time == 10000, 1002);
        let body = NFT::borrow_body(&card);
        let level = Card::get_level(body);
        assert!(level == 2, 1003);
        NFTGallery::deposit(&sender, card);
    }
}

// check: EXECUTED


//# run --signers creator
script {
    use creator::Card::{Self, Card};
    use StarcoinFramework::NFT;

    fun main(sender: signer) {
        let type_meta = NFT::nft_type_info_meta<Card>();
        assert!(NFT::meta_name(&type_meta) == b"", 1004);
        assert!(NFT::meta_image(&type_meta) == b"", 1005);
        assert!(NFT::meta_description(&type_meta) == b"", 1006);

        let name = b"card";
        let image = b"ipfs://image_hash";
        let description = b"a card game nft";
        let new_meta = NFT::new_meta_with_image(*&name, *&image, *&description);

        Card::update_type_info_meta(&sender, new_meta);

        let type_meta = NFT::nft_type_info_meta<Card>();
        assert!(NFT::meta_name(&type_meta) == name, 1007);
        assert!(NFT::meta_image(&type_meta) == image, 1008);
        assert!(NFT::meta_description(&type_meta) == description, 1009);
    }
}

// check: EXECUTED