address 0x1 {
module NFT {
    use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::CoreAddresses;
    use 0x1::Account;

    const ERR_NO_MINT_CAPABILITY: u64 = 101;
    const ERR_NO_BURN_CAPABILITY: u64 = 102;
    
    /// The info of NFT type
    struct NFTTypeInfo<NFTType: store + drop> has key, store {
        counter: u64,
    }

    struct GenesisSignerCapability has key {
        cap: Account::SignerCapability,
    }

    struct MintCapability<NFTType: store> has key, store {
    }

    struct BurnCapability<NFTType: store> has key, store {
    }

    struct NFT<NFTType: store + drop> has key, store {
        /// User specific Token of the NFT
        token: NFTType,
        /// The creator of NFT
        creator: address,
        /// The the unique id of NFT under NFTType
        uid: u64,
        /// The hash of the NFT content
        hash: vector<u8>,
    }

    /// The information of NFT instance return by get_nft_info
    struct NFTInfo<NFTType: store + drop> has drop {
        uid: u64,
        hash: vector<u8>,
        creator: address,
    }

    public fun get_info<NFTType: store + drop>(nft: &NFT<NFTType>): NFTInfo<NFTType> {
        return NFTInfo<NFTType> { uid: nft.uid, hash: *&nft.hash, creator: nft.creator }
    }

    public fun get_uid<NFTType: store + drop>(nft: &NFT<NFTType>): u64 {
        return nft.uid
    }

    public fun get_hash<NFTType: store + drop>(nft: &NFT<NFTType>): vector<u8> {
        return *&nft.hash
    }

    public fun get_creator<NFTType: store + drop>(nft: &NFT<NFTType>): address {
        return nft.creator
    }

    public fun initialize(signer: &signer) {
        CoreAddresses::assert_genesis_address(signer);
        let cap = Account::remove_signer_capability(signer);
        let genesis_cap = GenesisSignerCapability { cap };
        move_to(signer, genesis_cap);
    }

    /// Register a NFT type to genesis
    public fun register_nft<NFTType: store + drop>(signer: &signer) acquires GenesisSignerCapability {
        let genesis_cap = borrow_global<GenesisSignerCapability>(CoreAddresses::GENESIS_ADDRESS());
        let genesis_account = Account::create_signer_with_cap(&genesis_cap.cap);
        let info = NFTTypeInfo {
            counter: 0,
        };
        move_to<NFTTypeInfo<NFTType>>(&genesis_account, info);
        move_to<MintCapability<NFTType>>(signer, MintCapability {});
        move_to<BurnCapability<NFTType>>(signer, BurnCapability {});
    }

    /// Add MintCapability to `sender`
    public fun add_mint_capability<NFTType: store + drop>(sender: &signer, cap: MintCapability<NFTType>){
        move_to(sender, cap);
    }

    /// Remove the MintCapability<NFTType> from `sender`
    public fun remove_mint_capability<NFTType: store + drop>(sender: &signer): MintCapability<NFTType> acquires MintCapability {
        let addr = Signer::address_of(sender);
        assert(exists<MintCapability<NFTType>>(addr), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        move_from<MintCapability<NFTType>>(addr)
    }

    ///Destroy the MintCapability<NFTType>
    public fun destroy_mint_capability<NFTType: store + drop>(cap: MintCapability<NFTType>){
        let MintCapability{} = cap;
    }

    /// Mint nft with MintCapability<NFTTYpe>, `sender` will been the NFT's creator.
    public fun mint_with_cap<NFTType: store + drop>(sender: &signer, _cap: &mut MintCapability<NFTType>, hash: vector<u8>, token: NFTType): NFT<NFTType> acquires NFTTypeInfo {
        let creator = Signer::address_of(sender);
        let nft_type_info = borrow_global_mut<NFTTypeInfo<NFTType>>(CoreAddresses::GENESIS_ADDRESS());
        nft_type_info.counter = nft_type_info.counter + 1;
        let uid = nft_type_info.counter;
        let nft = NFT<NFTType> {
            token: token,
            uid: uid,
            hash: copy hash,
            creator,
        };
        return nft
    }

    /// Mint nft, the `sender` must have MintCapability<NFTType>
    public fun mint<NFTType: store + drop>(sender: &signer, hash: vector<u8>, token: NFTType): NFT<NFTType> acquires NFTTypeInfo, MintCapability {
        let addr = Signer::address_of(sender);
        assert(exists<MintCapability<NFTType>>(addr), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        let cap = borrow_global_mut<MintCapability<NFTType>>(addr);
        mint_with_cap(sender, cap, hash, token)
    }

    /// Add BurnCapability<NFTType> to `sender`
    public fun add_burn_capability<NFTType: store + drop>(sender: &signer, cap: BurnCapability<NFTType>){
        move_to(sender, cap);
    }

    /// Remove the BurnCapability<NFTType> from `sender`
    public fun remove_burn_capability<NFTType: store + drop>(sender: &signer): BurnCapability<NFTType> acquires BurnCapability {
        let addr = Signer::address_of(sender);
        assert(exists<BurnCapability<NFTType>>(addr), Errors::requires_capability(ERR_NO_BURN_CAPABILITY));
        move_from<BurnCapability<NFTType>>(addr)
    }

    ///Destroy the BurnCapability<NFTType>
    public fun destroy_burn_capability<NFTType: store + drop>(cap: BurnCapability<NFTType>){
        let BurnCapability{} = cap;
    }

    /// Burn nft with BurnCapability<NFTType>
    public fun burn_with_cap<NFTType: store +drop>(_cap: &mut BurnCapability<NFTType>, nft: NFT<NFTType>): NFTType {
        let NFT{ token,creator:_,uid:_,hash:_} = nft;
        token
    }

    /// Burn nft, the `sender` must have BurnCapability<NFTType>
    public fun burn<NFTType: store +drop>(sender: &signer, nft: NFT<NFTType>): NFTType acquires BurnCapability {
        let addr = Signer::address_of(sender);
        assert(exists<BurnCapability<NFTType>>(addr), Errors::requires_capability(ERR_NO_BURN_CAPABILITY));
        let cap = borrow_global_mut<BurnCapability<NFTType>>(addr);
        burn_with_cap(cap, nft)
    }
}

module NFTGallery {
    use 0x1::Collection2;
    use 0x1::Signer;
    use 0x1::NFT::{Self, NFT};
    use 0x1::Option::{Self, Option};
    use 0x1::Event;

    struct CreateEvent<NFTType: store + drop> has drop, store {
        uid: u64,
        hash: vector<u8>,
        creator: address,
    }

    struct TransferEvent<NFTType: store + drop> has drop, store {
        from: address,
        to: address,
        uid: u64,
    }

    struct NFTGallery<NFTType: store + drop> has key, store {
        create_events: Event::EventHandle<CreateEvent<NFTType>>,
        transfer_events: Event::EventHandle<TransferEvent<NFTType>>,
    }

    /// Init a NFTGallery to collect NFTs
    public fun init<NFTType: store + drop>(signer: &signer) {
        let gallery = NFTGallery {
            create_events: Event::new_event_handle<CreateEvent<NFTType>>(signer),
            transfer_events: Event::new_event_handle<TransferEvent<NFTType>>(signer),
        };
        move_to<NFTGallery<NFTType>>(signer, gallery);
        let address = Signer::address_of(signer);
        if (!Collection2::exists_at<NFT<NFTType>>(address)) {
            Collection2::create_collection<NFT<NFTType>>(signer, false, false);
        };
    }

    /// Create a NFT under the signer
    public fun create_nft<NFTType: store + drop>(signer: &signer, hash: vector<u8>, nft_type: NFTType) acquires NFTGallery {
        let address = Signer::address_of(signer);
        let gallery = borrow_global_mut<NFTGallery<NFTType>>(address);

        let nft = NFT::mint<NFTType>(signer, hash, nft_type);
        Event::emit_event(&mut gallery.create_events, CreateEvent<NFTType> {
            uid: NFT::get_uid(&nft),
            hash: NFT::get_hash(&nft),
            creator: NFT::get_creator(&nft)
        });
        Collection2::put(signer, address, nft);
    }

    /// Transfer NFT from signer to reciver
    public fun transfer_nft<NFTType: store + drop>(signer: &signer, uid: u64, receiver: address) acquires NFTGallery {
        let address = Signer::address_of(signer);
        let gallery = borrow_global_mut<NFTGallery<NFTType>>(address);
        let nfts = Collection2::borrow_collection<NFT<NFTType>>(signer, address);
        let i = 0;
        let len = Collection2::length(&nfts);
        // TODO: cache it?
        while (i < len) {
            if (&NFT::get_uid(Collection2::borrow(&nfts, i)) == &uid) break;
            i = i + 1;
        };
        let nft = Collection2::remove<NFT<NFTType>>(&mut nfts, i);
        Collection2::return_collection(nfts);
        Event::emit_event(&mut gallery.transfer_events, TransferEvent<NFTType> { from: address, to: receiver, uid: NFT::get_uid(&nft) });
        Collection2::put(signer, receiver, nft);
    }

    /// Get the NFT info
    public fun get_nft_info<NFTType: store + drop>(account: &signer, uid: u64): Option<NFT::NFTInfo<NFTType>> {
        let nfts = Collection2::borrow_collection<NFT<NFTType>>(account, Signer::address_of(account));
        let i = 0;
        let len = Collection2::length(&nfts);
        //TODO: cache it?
        while (i < len) {
            if (&NFT::get_uid(Collection2::borrow(&nfts, i)) == &uid) break;
            i = i + 1;
        };
        let nft = if (i != len) {
            let nft = Collection2::borrow<NFT<NFTType>>(&mut nfts, i);
            Option::some(NFT::get_info(nft))
        } else {
            Option::none<NFT::NFTInfo<NFTType>>()
        };
        Collection2::return_collection(nfts);
        return nft
    }

    public fun accept<NFTType: store + drop>(account: &signer) {
        Collection2::accept<NFT<NFTType>>(account);
    }

    //TODO should another method name?
    /// Deposit nft to `sender` NFTGallery
    public fun deposit<NFTType: store + drop>(sender: &signer, nft:NFT<NFTType>){
        Collection2::put(sender, Signer::address_of(sender), nft);
    }

    /// Deposit nft to `receiver` NFTGallery
    public fun deposit_to<NFTType: store + drop>(sender: &signer, receiver: address, nft:NFT<NFTType>){
        Collection2::put(sender, receiver, nft);
    }

    /// Withdraw one nft of NFTType from `sender`
    public fun withdraw_one<NFTType: store + drop>(sender: &signer): NFT<NFTType>{
        Collection2::take<NFT<NFTType>>(sender)
    }

    /// Withdraw nft of NFTType and uid from `sender`
    public fun withdraw<NFTType: store + drop>(_sender: &signer, _uid: u64){
        //TODO
    }

    /// Count all NFTs assigned to an owner
    public fun count_of<NFTType: store + drop>(owner: address):u64 {
        Collection2::length_of<NFT<NFTType>>(owner)
    }

}
}