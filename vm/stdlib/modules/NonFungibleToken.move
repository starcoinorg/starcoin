address 0x1 {
module NFT {
    use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::CoreAddresses;
    use 0x1::Account;

    const ERR_NO_MINT_CAPABILITY: u64 = 101;

    struct NFTTypeInfo<NFTType> has key, store {
        counter: u64,
        total_supply: u64,
    }

    struct GenesisSignerCapability has key {
        cap: Account::SignerCapability,
    }

    struct MintCapability<NFTType> has key, store {
        address: address,
    }

    struct NFTInfo has key, store, drop, copy {
        uid: u64,
        hash: vector<u8>,
        creator: address,
    }

    public fun get_info<NFTType: store>(nft: &NFT<NFTType>): NFTInfo {
        return NFTInfo { uid: nft.uid, hash: *&nft.hash, creator: nft.creator }
    }

    public fun get_uid<NFTType: store>(nft: &NFT<NFTType>): u64 {
        return nft.uid
    }

    struct NFT<NFTType: store> has key, store {
        token: NFTType,
        creator: address,
        uid: u64,
        hash: vector<u8>,
    }

    public fun initialize(signer: &signer) {
        CoreAddresses::assert_genesis_address(signer);
        let cap = Account::remove_signer_capability(signer);
        let genesis_cap = GenesisSignerCapability { cap };
        move_to(signer, genesis_cap);
    }


    public fun register_nft<NFTType: store>(signer: &signer, total_supply: u64) acquires GenesisSignerCapability {
        let genesis_cap = borrow_global<GenesisSignerCapability>(CoreAddresses::GENESIS_ADDRESS());
        let genesis_account = Account::create_signer_with_cap(&genesis_cap.cap);
        let info = NFTTypeInfo {
            counter: 0,
            total_supply: total_supply,
        };
        move_to<NFTTypeInfo<NFTType>>(&genesis_account, info);
        move_to<MintCapability<NFTType>>(signer, MintCapability { address: Signer::address_of(signer) });
    }


    public fun remove_mint_capability<NFTType: store>(signer: &signer): MintCapability<NFTType> acquires MintCapability {
        let account = Signer::address_of(signer);
        assert(exists<MintCapability<NFTType>>(account), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        move_from<MintCapability<NFTType>>(account)
    }

    public fun mint<NFTType: store>(account: &signer, hash: vector<u8>, token: NFTType): NFT<NFTType> acquires NFTTypeInfo {
        let address = Signer::address_of(account);
        assert(exists<MintCapability<NFTType>>(address), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        let nft_type_info = borrow_global_mut<NFTTypeInfo<NFTType>>(CoreAddresses::GENESIS_ADDRESS());
        nft_type_info.counter = nft_type_info.counter + 1;
        let uid = nft_type_info.counter;
        let nft = NFT<NFTType> {
            token: token,
            uid: uid,
            hash: hash,
            creator: address,
        };
        return nft
    }
}

module NFTGallery {
    use 0x1::Collection2;
    use 0x1::Signer;
    use 0x1::NFT::{Self, NFT};
    use 0x1::Option::{Self, Option};

    public fun init<NFTType: store>(account: &signer) {
        let address = Signer::address_of(account);
        if (!Collection2::exists_at<NFT<NFTType>>(address)) {
            Collection2::create_collection<NFT<NFTType>>(account, false, false);
        };
    }

    public fun create_nft<NFTType: store>(account: &signer, hash: vector<u8>, nft_type: NFTType) {
        let address = Signer::address_of(account);
        let nft = NFT::mint<NFTType>(account, hash, nft_type);
        Collection2::put(account, address, nft);
    }

    public fun transfer_nft<NFTType: store>(account: &signer, uid: u64, receiver: address) {
        let address = Signer::address_of(account);
        let nfts = Collection2::borrow_collection<NFT<NFTType>>(account, address);
        let i = 0;
        let len = Collection2::length(&nfts);
        while (i < len) {
            if (&NFT::get_uid(Collection2::borrow(&nfts, i)) == &uid) break;
            i = i + 1;
        };
        let nft = Collection2::remove<NFT<NFTType>>(&mut nfts, i);
        Collection2::return_collection(nfts);
        Collection2::put(account, receiver, nft);
    }

    public fun get_nft<NFTType: store>(account: &signer, uid: u64): Option<NFT::NFTInfo> {
        let nfts = Collection2::borrow_collection<NFT<NFTType>>(account, Signer::address_of(account));
        let i = 0;
        let len = Collection2::length(&nfts);
        while (i < len) {
            if (&NFT::get_uid(Collection2::borrow(&nfts, i)) == &uid) break;
            i = i + 1;
        };
        let nft = if (i != len) {
            let nft = Collection2::borrow<NFT<NFTType>>(&mut nfts, i);
            Option::some(NFT::get_info(nft))
        } else {
            Option::none<NFT::NFTInfo>()
        };
        Collection2::return_collection(nfts);
        return nft
    }

    public fun accept<NFTType: store>(account: &signer) {
        Collection2::accept<NFT<NFTType>>(account);
    }
}
}

