//# init -n dev

//# faucet --addr creator

//# faucet --addr bob

//# publish
module creator::Card {
    use StarcoinFramework::NFT::{Self, NFT, MintCapability, BurnCapability};
    use StarcoinFramework::Timestamp;

    struct L1CardMeta has copy, store, drop{
        gene: u64,
    }
    struct L2CardMeta has copy, store, drop{
        gene: u64,
    }

    struct L1Card has store {}
    struct L2Card has store {
        first: L1Card,
        second: L1Card,
    }

    struct L1CardMintCapability has key{
        cap: MintCapability<L1CardMeta>,
    }

    struct L2CardMintCapability has key{
        cap: MintCapability<L2CardMeta>,
    }

    struct L1CardBurnCapability has key{
        cap: BurnCapability<L1CardMeta>,
    }

    struct L2CardBurnCapability has key{
        cap: BurnCapability<L2CardMeta>,
    }

    public fun init(sender: &signer){
        NFT::register_v2<L1CardMeta>(sender, NFT::empty_meta());
        let cap = NFT::remove_mint_capability<L1CardMeta>(sender);
        move_to(sender, L1CardMintCapability{ cap});

        let cap = NFT::remove_burn_capability<L1CardMeta>(sender);
        move_to(sender, L1CardBurnCapability{ cap});
        NFT::register_v2<L2CardMeta>(sender, NFT::empty_meta());
        let cap = NFT::remove_mint_capability<L2CardMeta>(sender);
        move_to(sender, L2CardMintCapability{ cap});

        let cap = NFT::remove_burn_capability<L2CardMeta>(sender);
        move_to(sender, L2CardBurnCapability{ cap});
    }

    public fun mint_l1(_sender: &signer): NFT<L1CardMeta, L1Card> acquires L1CardMintCapability{
        let cap = borrow_global_mut<L1CardMintCapability>(@creator);
        let metadata = NFT::new_meta_with_image(b"l1_card", b"ipfs:://xxxxxx", b"This is a L1CardMeta nft.");
        NFT::mint_with_cap_v2<L1CardMeta, L1Card>(@creator, &mut cap.cap, metadata, L1CardMeta{ gene: Timestamp::now_milliseconds()}, L1Card{})
    }

    public fun mint_l2(_sender: &signer, first: NFT<L1CardMeta, L1Card>, second: NFT<L1CardMeta, L1Card>): NFT<L2CardMeta,L2Card> acquires L1CardBurnCapability, L2CardMintCapability {
        let burn_cap = borrow_global_mut<L1CardBurnCapability>(@creator);
        let new_gene = {
            let first_meta = NFT::get_type_meta(&first);
            let second_meta = NFT::get_type_meta(&second);
            first_meta.gene + second_meta.gene
        };
        let f = NFT::burn_with_cap(&mut burn_cap.cap, first);
        let s = NFT::burn_with_cap(&mut burn_cap.cap, second);
        let mint_cap = borrow_global_mut<L2CardMintCapability>(@creator);
        let metadata = NFT::new_meta_with_image(b"l2_card", b"ipfs:://xxxxxx", b"This is a L2CardMeta nft.");
        NFT::mint_with_cap_v2<L2CardMeta, L2Card>(@creator, &mut mint_cap.cap, metadata, L2CardMeta{
            gene: new_gene,
        }, L2Card{
            first:f,
            second:s,
        })
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
        let first_l1 = Card::mint_l1(&sender);
        NFTGallery::deposit(&sender, first_l1);
    }
}

// check: EXECUTED

//# run --signers bob
script {
    use StarcoinFramework::NFTGallery;
    use creator::Card::{Self, L1CardMeta, L1Card};
    use StarcoinFramework::Signer;

    fun main(sender: signer) {
        let second_l1 = Card::mint_l1(&sender);
        NFTGallery::deposit(&sender, second_l1);
        assert!(NFTGallery::count_of<L1CardMeta, L1Card>(Signer::address_of(&sender)) == 2, 1000);
    }
}

// check: EXECUTED


//# run --signers bob
script {
    use StarcoinFramework::NFTGallery;
    use creator::Card::{Self, L1CardMeta, L1Card};
    fun main(sender: signer) {
        let first_l1 = NFTGallery::withdraw_one<L1CardMeta, L1Card>(&sender);
        let second_l1 = NFTGallery::withdraw_one<L1CardMeta, L1Card>(&sender);
        let l2_card = Card::mint_l2(&sender, first_l1, second_l1);
        NFTGallery::deposit(&sender, l2_card);
    }
}

// check: EXECUTED

//# run --signers bob
script {
    use creator::Card::{L1CardMeta, L2CardMeta, L1Card, L2Card};
    use StarcoinFramework::NFTGallery;
    use StarcoinFramework::Signer;

    fun main(sender: signer) {
        assert!(NFTGallery::count_of<L1CardMeta, L1Card>(Signer::address_of(&sender)) == 0, 1001);
        assert!(NFTGallery::count_of<L2CardMeta, L2Card>(Signer::address_of(&sender)) == 1, 1002);
    }
}

// check: EXECUTED