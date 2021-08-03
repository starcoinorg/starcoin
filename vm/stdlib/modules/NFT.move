address 0x1 {
///Non-fungible token standard and implements.
module NFT {
    use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::CoreAddresses;
    use 0x1::Account;
    use 0x1::Vector;
    use 0x1::Event;

    const ERR_NO_MINT_CAPABILITY: u64 = 101;
    const ERR_NO_BURN_CAPABILITY: u64 = 102;

    struct MintEvent<NFTType: store + drop> has drop, store {
        uid: u64,
        creator: address,
        metadata: Metadata,
    }

    /// The info of NFT type
    struct NFTTypeInfo<NFTType: store + drop> has key, store {
        counter: u64,
        mint_events: Event::EventHandle<MintEvent<NFTType>>,
    }

    struct GenesisSignerCapability has key {
        cap: Account::SignerCapability,
    }

    struct MintCapability<NFTType: store> has key, store {
    }

    struct BurnCapability<NFTType: store> has key, store {
    }

    struct Attribute has copy, store, drop{
        name: vector<u8>,
        value: vector<u8>,
    }

    struct Metadata has copy, store, drop{
        name: vector<u8>,
        image: vector<u8>,
        image_data: vector<u8>,
        description: vector<u8>,
        attributes: vector<Attribute>,
    }

    public fun new_metadata(name: vector<u8>, image: vector<u8>, description: vector<u8>): Metadata{
        Metadata{
            name,
            image,
            image_data: Vector::empty(),
            description,
            attributes: Vector::empty(),
        }
    }

    public fun new_metadata_with_image_data(name: vector<u8>, image_data: vector<u8>, description: vector<u8>): Metadata{
        Metadata{
            name,
            image: Vector::empty(),
            image_data,
            description,
            attributes: Vector::empty(),
        }
    }

    public fun metadata_name(metadata: &Metadata):vector<u8>{
        *&metadata.name
    }

    public fun metadata_image(metadata: &Metadata):vector<u8>{
        *&metadata.image
    }

    public fun metadata_image_data(metadata: &Metadata):vector<u8>{
        *&metadata.image_data
    }

    public fun metadata_description(metadata: &Metadata):vector<u8>{
        *&metadata.description
    }

    public fun metadata_attributes(metadata: &Metadata):&vector<Attribute>{
        &metadata.attributes
    }

    public fun metadata_attributes_mut(metadata: &mut Metadata):&mut vector<Attribute>{
        &mut metadata.attributes
    }

    public fun append_metadata_attributes(metadata: &mut Metadata, attribute: Attribute){
        Vector::push_back(&mut metadata.attributes, attribute);
    }

    struct NFT<NFTType: store + drop> has store {
        /// User specific Token of the NFT
        token: NFTType,
        /// The creator of NFT
        creator: address,
        /// The the unique id of NFT under NFTType
        uid: u64,
        /// metadata
        metadata: Metadata,
    }

    /// The information of NFT instance return by get_nft_info
    struct NFTInfo<NFTType: store + drop> has copy, store, drop {
        uid: u64,
        creator: address,
        metadata: Metadata,
    }

    public fun get_info<NFTType: store + drop>(nft: &NFT<NFTType>): NFTInfo<NFTType> {
        return NFTInfo<NFTType> { uid: nft.uid, creator: nft.creator, metadata: *&nft.metadata }
    }

    public fun get_uid<NFTType: store + drop>(nft: &NFT<NFTType>): u64 {
        return nft.uid
    }

    public fun get_metadata<NFTType: store + drop>(nft: &NFT<NFTType>): Metadata {
        return *&nft.metadata
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
    public fun register<NFTType: store + drop>(sender: &signer) acquires GenesisSignerCapability {
        let genesis_cap = borrow_global<GenesisSignerCapability>(CoreAddresses::GENESIS_ADDRESS());
        let genesis_account = Account::create_signer_with_cap(&genesis_cap.cap);
        let info = NFTTypeInfo {
            counter: 0,
            mint_events: Event::new_event_handle<MintEvent<NFTType>>(sender),
        };
        move_to<NFTTypeInfo<NFTType>>(&genesis_account, info);
        move_to<MintCapability<NFTType>>(sender, MintCapability {});
        move_to<BurnCapability<NFTType>>(sender, BurnCapability {});
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
    public fun mint_with_cap<NFTType: store + drop>(sender: &signer, _cap: &mut MintCapability<NFTType>, metadata: Metadata, token: NFTType): NFT<NFTType> acquires NFTTypeInfo {
        let creator = Signer::address_of(sender);
        let nft_type_info = borrow_global_mut<NFTTypeInfo<NFTType>>(CoreAddresses::GENESIS_ADDRESS());
        nft_type_info.counter = nft_type_info.counter + 1;
        let uid = nft_type_info.counter;
        let nft = NFT<NFTType> {
            token: token,
            uid: uid,
            creator,
            metadata: copy metadata,
        };
        Event::emit_event(&mut nft_type_info.mint_events, MintEvent<NFTType> {
            uid,
            creator,
            metadata,
        });
        return nft
    }

    /// Mint nft, the `sender` must have MintCapability<NFTType>
    public fun mint<NFTType: store + drop>(sender: &signer,  metadata: Metadata, token: NFTType): NFT<NFTType> acquires NFTTypeInfo, MintCapability {
        let addr = Signer::address_of(sender);
        assert(exists<MintCapability<NFTType>>(addr), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        let cap = borrow_global_mut<MintCapability<NFTType>>(addr);
        mint_with_cap(sender, cap, metadata, token)
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
        let NFT{ token,creator:_,uid:_,metadata:_} = nft;
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
/// NFTGallery is user collection of NFT.
module NFTGallery {
    use 0x1::Collection2::{Self,Collection};
    use 0x1::Signer;
    use 0x1::NFT::{Self, NFT};
    use 0x1::Option::{Self, Option};
    use 0x1::Event;
    use 0x1::Errors;

    const ERR_NFT_NOT_EXISTS: u64 = 101;

    struct WithdrawEvent<NFTType: store + drop> has drop, store {
        uid: u64,
    }

    struct DepositEvent<NFTType: store + drop> has drop, store {
        uid: u64,
    }

    struct NFTGallery<NFTType: store + drop> has key, store {
        withdraw_events: Event::EventHandle<WithdrawEvent<NFTType>>,
        deposit_events: Event::EventHandle<DepositEvent<NFTType>>,
    }

    /// Init a NFTGallery to accept NFTType
    public fun accept<NFTType: store + drop>(sender: &signer) {
        let gallery = NFTGallery {
            withdraw_events: Event::new_event_handle<WithdrawEvent<NFTType>>(sender),
            deposit_events: Event::new_event_handle<DepositEvent<NFTType>>(sender),
        };
        move_to<NFTGallery<NFTType>>(sender, gallery);
        Collection2::accept<NFT<NFTType>>(sender);
    }

    /// Transfer NFT from `sender` to `receiver`
    public fun transfer<NFTType: store + drop>(sender: &signer, uid: u64, receiver: address) acquires NFTGallery {
        let nft = withdraw<NFTType>(sender, uid);
        assert(Option::is_some(&nft), Errors::not_published(ERR_NFT_NOT_EXISTS));
        let nft = Option::destroy_some(nft);
        deposit_to(sender, receiver, nft)
    }

    /// Get the NFT info
    public fun get_nft_info<NFTType: store + drop>(account: &signer, uid: u64): Option<NFT::NFTInfo<NFTType>> {
        let nfts = Collection2::borrow_collection<NFT<NFTType>>(account, Signer::address_of(account));
        let idx = find_by_uid<NFTType>(&nfts, uid);

        let info = if (Option::is_some(&idx)) {
            let i = Option::extract(&mut idx);
            let nft = Collection2::borrow<NFT<NFTType>>(&mut nfts, i);
            Option::some(NFT::get_info(nft))
        } else {
            Option::none<NFT::NFTInfo<NFTType>>()
        };
        Collection2::return_collection(nfts);
        return info
    }

    /// Deposit nft to `sender` NFTGallery
    public fun deposit<NFTType: store + drop>(sender: &signer, nft:NFT<NFTType>) acquires NFTGallery{
        deposit_to(sender, Signer::address_of(sender), nft)
    }

    /// Deposit nft to `receiver` NFTGallery
    public fun deposit_to<NFTType: store + drop>(sender: &signer, receiver: address, nft:NFT<NFTType>) acquires NFTGallery{
        let gallery = borrow_global_mut<NFTGallery<NFTType>>(receiver);
        Event::emit_event(&mut gallery.deposit_events, DepositEvent<NFTType> { uid: NFT::get_uid(&nft) });
        Collection2::put(sender, receiver, nft);
    }

    /// Withdraw one nft of NFTType from `sender`
    public fun withdraw_one<NFTType: store + drop>(sender: &signer): Option<NFT<NFTType>> acquires NFTGallery{
        do_withdraw<NFTType>(sender, Option::none())
    }

    /// Withdraw nft of NFTType and uid from `sender`
    public fun withdraw<NFTType: store + drop>(sender: &signer, uid: u64) : Option<NFT<NFTType>> acquires NFTGallery{
       do_withdraw(sender, Option::some(uid))
    }

    /// Withdraw nft of NFTType and uid from `sender`
    fun do_withdraw<NFTType: store + drop>(sender: &signer, uid: Option<u64>) : Option<NFT<NFTType>> acquires NFTGallery{
        let sender_addr = Signer::address_of(sender);
        let gallery = borrow_global_mut<NFTGallery<NFTType>>(sender_addr);
        let nfts = Collection2::borrow_collection<NFT<NFTType>>(sender, sender_addr);
        let len = Collection2::length(&nfts);
        let nft = if(len == 0){
            Option::none()
        }else{
            let idx = if (Option::is_some(&uid)){
                let uid = Option::extract(&mut uid);
                find_by_uid(&nfts, uid)
            }else{
                //default withdraw the last nft.
                Option::some(len -1)
            };

            if (Option::is_some(&idx)){
                let i = Option::extract(&mut idx);
                let nft = Collection2::remove<NFT<NFTType>>(&mut nfts, i);
                Event::emit_event(&mut gallery.withdraw_events, WithdrawEvent<NFTType> { uid: NFT::get_uid(&nft) });
                Option::some(nft)
            }else{
                Option::none()
            }
        };
        Collection2::return_collection(nfts);
        nft
    }

    fun find_by_uid<NFTType: store + drop>(c: &Collection<NFT<NFTType>>, uid: u64): Option<u64>{
        let len = Collection2::length(c);
        if(len == 0){
            return Option::none()
        };
        let idx = len - 1;
        loop {
            let nft = Collection2::borrow(c, idx);
            if (NFT::get_uid(nft) == uid){
                return Option::some(idx)
            };
            if(idx == 0){
                return Option::none()
            };
            idx = idx - 1;
        }
    }

    /// Count all NFTs assigned to an owner
    public fun count_of<NFTType: store + drop>(owner: address):u64 {
        Collection2::length_of<NFT<NFTType>>(owner)
    }

}
}