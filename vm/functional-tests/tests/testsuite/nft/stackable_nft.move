//! account: creator
//! account: bob

//! sender: creator
address creator = {{creator}};
module creator::StackableNFT {
    use 0x1::NFT::{Self, NFT, Metadata, MintCapability, BurnCapability, UpdateCapability};

    struct NFTQuantity has store {
        value: u64,
    }

    struct StackableNFTInfo has copy, store, drop{
        total_amount: u64,
    }

    public fun register<NFTMeta: copy + store + drop>(sender: &signer, total_amount: u64, meta: Metadata) {
        NFT::register<NFTMeta, StackableNFTInfo>(sender, StackableNFTInfo{ total_amount}, meta);
    }

    public fun mint_with_cap<NFTMeta: copy + store + drop>
        (creator: address, cap: &mut MintCapability<NFTMeta>, type_meta: NFTMeta, quantity: u64): NFT<NFTMeta, NFTQuantity>{
        NFT::mint_with_cap<NFTMeta, NFTQuantity, StackableNFTInfo>(creator, cap, NFT::empty_meta(), type_meta, NFTQuantity{ value: quantity})
    }

    public fun burn_with_cap<NFTMeta: copy + store + drop>(cap: &mut BurnCapability<NFTMeta>, nft: NFT<NFTMeta, NFTQuantity>) {
        let body = NFT::burn_with_cap(cap, nft);
        let NFTQuantity{ value: _} = body;
    }

    /// Split nft to two NFT
    public fun split<NFTMeta: copy + store + drop>(update_cap: &mut UpdateCapability<NFTMeta>, mint_cap: &mut MintCapability<NFTMeta>, nft: &mut NFT<NFTMeta, NFTQuantity>, quantity: u64): NFT<NFTMeta, NFTQuantity> {
        let info = NFT::get_info(nft);
        let body = NFT::borrow_body_mut_with_cap(update_cap, nft);
        assert(body.value > quantity, 1000);
        let (_id,creator,_metadata,type_meta) = NFT::unpack_info(info);
        body.value = body.value - quantity;
        Self::mint_with_cap<NFTMeta>(creator, mint_cap, type_meta, quantity)
    }

    /// Merge other_nft to target_nft
    public fun merge<NFTMeta: copy + store + drop>(burn_cap: &mut BurnCapability<NFTMeta>, update_cap: &mut UpdateCapability<NFTMeta>, target_nft: &mut NFT<NFTMeta, NFTQuantity>, other_nft: NFT<NFTMeta, NFTQuantity>){
        let other_body = NFT::burn_with_cap(burn_cap, other_nft);
        let NFTQuantity{ value: quantity} = other_body;
        let body = NFT::borrow_body_mut_with_cap(update_cap, target_nft);
        body.value = body.value + quantity;
    }

}

// check: EXECUTED