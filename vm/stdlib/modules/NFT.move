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
    const ERR_NO_UPDATE_CAPABILITY: u64 = 103;
    const ERR_CANOT_EMPTY: u64 = 104;

    struct MintEvent<NFTMeta: copy + store + drop> has drop, store {
        uid: u64,
        creator: address,
        base_meta: Metadata,
        type_meta: NFTMeta,
    }

    /// The info of NFT type
    struct NFTTypeInfo<NFTMeta: copy + store + drop> has key, store {
        counter: u64,
        mint_events: Event::EventHandle<MintEvent<NFTMeta>>,
    }

    struct GenesisSignerCapability has key {
        cap: Account::SignerCapability,
    }
    /// The capability to mint the nft.
    struct MintCapability<NFTMeta: store> has key, store {
    }
    /// The Capability to burn the nft.
    struct BurnCapability<NFTMeta: store> has key, store {
    }
    /// The Capability to update the nft metadata.
    struct UpdateCapability<NFTMeta: store> has key, store {
    }

    struct Metadata has copy, store, drop{
        /// NFT name's utf8 bytes.
        name: vector<u8>,
        /// Image link, such as ipfs://xxxx
        image: vector<u8>,
        /// Image bytes data, image or image_data can not empty for both.
        image_data: vector<u8>,
        /// NFT description utf8 bytes.
        description: vector<u8>,
    }

    public fun new_meta_with_image(name: vector<u8>, image: vector<u8>, description: vector<u8>): Metadata{
        assert(!Vector::is_empty(&name), Errors::invalid_argument(ERR_CANOT_EMPTY));
        assert(!Vector::is_empty(&image), Errors::invalid_argument(ERR_CANOT_EMPTY));
        Metadata{
            name,
            image,
            image_data: Vector::empty(),
            description,
        }
    }

    public fun new_meta_with_image_data(name: vector<u8>, image_data: vector<u8>, description: vector<u8>): Metadata{
        assert(!Vector::is_empty(&name), Errors::invalid_argument(ERR_CANOT_EMPTY));
        assert(!Vector::is_empty(&image_data), Errors::invalid_argument(ERR_CANOT_EMPTY));
        Metadata{
            name,
            image: Vector::empty(),
            image_data,
            description,
        }
    }

    public fun meta_name(metadata: &Metadata):vector<u8>{
        *&metadata.name
    }

    public fun meta_image(metadata: &Metadata):vector<u8>{
        *&metadata.image
    }

    public fun meta_image_data(metadata: &Metadata):vector<u8>{
        *&metadata.image_data
    }

    public fun meta_description(metadata: &Metadata):vector<u8>{
        *&metadata.description
    }

    struct NFT<NFTMeta: copy + store + drop, NFTBody> has store {
        /// The creator of NFT
        creator: address,
        /// The the unique id of NFT under NFTMeta type
        uid: u64,
        /// The metadata of NFT
        base_meta: Metadata,
        /// The extension metadata of NFT
        type_meta: NFTMeta,
        /// The body of NFT, NFT is a box for NFTBody
        body: NFTBody,
    }

    /// The information of NFT instance return by get_nft_info
    struct NFTInfo<NFTMeta: copy + store + drop> has copy, store, drop {
        uid: u64,
        creator: address,
        base_meta: Metadata,
        type_meta: NFTMeta,
    }

    public fun get_info<NFTMeta: copy + store + drop, NFTBody: store>(nft: &NFT<NFTMeta, NFTBody>): NFTInfo<NFTMeta> {
        return NFTInfo<NFTMeta> { uid: nft.uid, creator: nft.creator, base_meta: *&nft.base_meta, type_meta:*&nft.type_meta }
    }

    public fun get_uid<NFTMeta: copy + store + drop, NFTBody: store>(nft: &NFT<NFTMeta, NFTBody>): u64 {
        return nft.uid
    }

    public fun get_base_meta<NFTMeta: copy + store + drop, NFTBody: store>(nft: &NFT<NFTMeta, NFTBody>): &Metadata {
        return &nft.base_meta
    }

    public fun get_type_meta<NFTMeta: copy + store + drop, NFTBody: store>(nft: &NFT<NFTMeta, NFTBody>): &NFTMeta {
        return &nft.type_meta
    }

    public fun get_creator<NFTMeta: copy + store + drop, NFTBody: store>(nft: &NFT<NFTMeta, NFTBody>): address {
        return nft.creator
    }

    public fun initialize(signer: &signer) {
        CoreAddresses::assert_genesis_address(signer);
        let cap = Account::remove_signer_capability(signer);
        let genesis_cap = GenesisSignerCapability { cap };
        move_to(signer, genesis_cap);
    }

    /// Register a NFT type to genesis
    public fun register<NFTMeta: copy + store + drop>(sender: &signer) acquires GenesisSignerCapability {
        let genesis_cap = borrow_global<GenesisSignerCapability>(CoreAddresses::GENESIS_ADDRESS());
        let genesis_account = Account::create_signer_with_cap(&genesis_cap.cap);
        let info = NFTTypeInfo {
            counter: 0,
            mint_events: Event::new_event_handle<MintEvent<NFTMeta>>(sender),
        };
        move_to<NFTTypeInfo<NFTMeta>>(&genesis_account, info);
        move_to<MintCapability<NFTMeta>>(sender, MintCapability {});
        move_to<BurnCapability<NFTMeta>>(sender, BurnCapability {});
        move_to<UpdateCapability<NFTMeta>>(sender, UpdateCapability {});
    }

    /// Add MintCapability to `sender`
    public fun add_mint_capability<NFTMeta: copy + store + drop>(sender: &signer, cap: MintCapability<NFTMeta>){
        move_to(sender, cap);
    }

    /// Remove the MintCapability<NFTMeta> from `sender`
    public fun remove_mint_capability<NFTMeta: copy + store + drop>(sender: &signer): MintCapability<NFTMeta> acquires MintCapability {
        let addr = Signer::address_of(sender);
        assert(exists<MintCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        move_from<MintCapability<NFTMeta>>(addr)
    }

    ///Destroy the MintCapability<NFTMeta>
    public fun destroy_mint_capability<NFTMeta: copy + store + drop>(cap: MintCapability<NFTMeta>){
        let MintCapability{} = cap;
    }

    /// Mint nft with MintCapability<NFTTYpe>, `sender` will been the NFT's creator.
    public fun mint_with_cap<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, _cap: &mut MintCapability<NFTMeta>, base_meta: Metadata, type_meta: NFTMeta, body: NFTBody): NFT<NFTMeta, NFTBody> acquires NFTTypeInfo {
        let creator = Signer::address_of(sender);
        let nft_type_info = borrow_global_mut<NFTTypeInfo<NFTMeta>>(CoreAddresses::GENESIS_ADDRESS());
        nft_type_info.counter = nft_type_info.counter + 1;
        let uid = nft_type_info.counter;
        let nft = NFT<NFTMeta, NFTBody> {
            uid: uid,
            creator,
            base_meta: copy base_meta,
            type_meta: copy type_meta,
            body,
        };
        Event::emit_event(&mut nft_type_info.mint_events, MintEvent<NFTMeta> {
            uid,
            creator,
            base_meta,
            type_meta,
        });
        return nft
    }

    /// Mint nft, the `sender` must have MintCapability<NFTMeta>
    public fun mint<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer,  base_meta: Metadata, type_meta: NFTMeta, body: NFTBody): NFT<NFTMeta, NFTBody> acquires NFTTypeInfo, MintCapability {
        let addr = Signer::address_of(sender);
        assert(exists<MintCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        let cap = borrow_global_mut<MintCapability<NFTMeta>>(addr);
        mint_with_cap(sender, cap, base_meta, type_meta, body)
    }

    /// Add BurnCapability<NFTMeta> to `sender`
    public fun add_burn_capability<NFTMeta: copy + store + drop>(sender: &signer, cap: BurnCapability<NFTMeta>){
        move_to(sender, cap);
    }

    /// Remove the BurnCapability<NFTMeta> from `sender`
    public fun remove_burn_capability<NFTMeta: copy + store + drop>(sender: &signer): BurnCapability<NFTMeta> acquires BurnCapability {
        let addr = Signer::address_of(sender);
        assert(exists<BurnCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_BURN_CAPABILITY));
        move_from<BurnCapability<NFTMeta>>(addr)
    }

    ///Destroy the BurnCapability<NFTMeta>
    public fun destroy_burn_capability<NFTMeta: copy + store + drop>(cap: BurnCapability<NFTMeta>){
        let BurnCapability{} = cap;
    }

    /// Burn nft with BurnCapability<NFTMeta>
    public fun burn_with_cap<NFTMeta: copy + store + drop, NFTBody: store>(_cap: &mut BurnCapability<NFTMeta>, nft: NFT<NFTMeta, NFTBody>): NFTBody {
        let NFT{ creator:_,uid:_,base_meta:_, type_meta:_, body} = nft;
        body
    }

    /// Burn nft, the `sender` must have BurnCapability<NFTMeta>
    public fun burn<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, nft: NFT<NFTMeta, NFTBody>): NFTBody acquires BurnCapability {
        let addr = Signer::address_of(sender);
        assert(exists<BurnCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_BURN_CAPABILITY));
        let cap = borrow_global_mut<BurnCapability<NFTMeta>>(addr);
        burn_with_cap(cap, nft)
    }

    /// Add UpdateCapability<NFTMeta> to `sender`
    public fun add_update_capability<NFTMeta: copy + store + drop>(sender: &signer, cap: UpdateCapability<NFTMeta>){
        move_to(sender, cap);
    }

    /// Remove the BurnCapability<NFTMeta> from `sender`
    public fun remove_update_capability<NFTMeta: copy + store + drop>(sender: &signer): UpdateCapability<NFTMeta> acquires UpdateCapability {
        let addr = Signer::address_of(sender);
        assert(exists<UpdateCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        move_from<UpdateCapability<NFTMeta>>(addr)
    }

    ///Destroy the UpdateCapability<NFTMeta>
    public fun destroy_update_capability<NFTMeta: copy + store + drop>(cap: UpdateCapability<NFTMeta>){
        let UpdateCapability{} = cap;
    }

    /// Update the nft's base_meta and type_meta with UpdateCapability<NFTMeta>
    public fun update_meta_with_cap<NFTMeta: copy + store + drop, NFTBody: store>(_cap: &mut UpdateCapability<NFTMeta>, nft: &mut NFT<NFTMeta, NFTBody>, base_meta: Metadata, type_meta: NFTMeta) {
        nft.base_meta = base_meta;
        nft.type_meta = type_meta;
    }

    /// Update the nft's base_meta and type_meta, the `sender` must have UpdateCapability<NFTMeta>
    public fun update_meta<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, nft: &mut NFT<NFTMeta, NFTBody>, base_meta: Metadata, type_meta: NFTMeta) acquires UpdateCapability {
        let addr = Signer::address_of(sender);
        assert(exists<UpdateCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        let cap = borrow_global_mut<UpdateCapability<NFTMeta>>(addr);
        update_meta_with_cap(cap, nft, base_meta, type_meta)
    }

    /// Borrow NFTBody mut ref for update body with UpdateCapability<NFTMeta>
    public fun borrow_body_mut_with_cap<NFTMeta: copy + store + drop, NFTBody: store>(_cap: &mut UpdateCapability<NFTMeta>, nft: &mut NFT<NFTMeta, NFTBody>): &mut NFTBody{
        &mut nft.body
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

    struct WithdrawEvent<NFTMeta: copy + store + drop> has drop, store {
        uid: u64,
    }

    struct DepositEvent<NFTMeta: copy + store + drop> has drop, store {
        uid: u64,
    }

    struct NFTGallery<NFTMeta: copy + store + drop> has key, store {
        withdraw_events: Event::EventHandle<WithdrawEvent<NFTMeta>>,
        deposit_events: Event::EventHandle<DepositEvent<NFTMeta>>,
    }

    /// Init a NFTGallery to accept NFTMeta
    public fun accept<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer) {
        let gallery = NFTGallery {
            withdraw_events: Event::new_event_handle<WithdrawEvent<NFTMeta>>(sender),
            deposit_events: Event::new_event_handle<DepositEvent<NFTMeta>>(sender),
        };
        move_to<NFTGallery<NFTMeta>>(sender, gallery);
        Collection2::accept<NFT<NFTMeta, NFTBody>>(sender);
    }

    /// Transfer NFT from `sender` to `receiver`
    public fun transfer<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, uid: u64, receiver: address) acquires NFTGallery {
        let nft = withdraw<NFTMeta, NFTBody>(sender, uid);
        assert(Option::is_some(&nft), Errors::not_published(ERR_NFT_NOT_EXISTS));
        let nft = Option::destroy_some(nft);
        deposit_to(sender, receiver, nft)
    }

    /// Get the NFT info
    public fun get_nft_info<NFTMeta: copy + store + drop, NFTBody: store>(account: &signer, uid: u64): Option<NFT::NFTInfo<NFTMeta>> {
        let nfts = Collection2::borrow_collection<NFT<NFTMeta, NFTBody>>(account, Signer::address_of(account));
        let idx = find_by_uid<NFTMeta, NFTBody>(&nfts, uid);

        let info = if (Option::is_some(&idx)) {
            let i = Option::extract(&mut idx);
            let nft = Collection2::borrow<NFT<NFTMeta, NFTBody>>(&mut nfts, i);
            Option::some(NFT::get_info(nft))
        } else {
            Option::none<NFT::NFTInfo<NFTMeta>>()
        };
        Collection2::return_collection(nfts);
        return info
    }

    /// Deposit nft to `sender` NFTGallery
    public fun deposit<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, nft:NFT<NFTMeta, NFTBody>) acquires NFTGallery{
        deposit_to(sender, Signer::address_of(sender), nft)
    }

    /// Deposit nft to `receiver` NFTGallery
    public fun deposit_to<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, receiver: address, nft:NFT<NFTMeta,NFTBody>) acquires NFTGallery{
        let gallery = borrow_global_mut<NFTGallery<NFTMeta>>(receiver);
        Event::emit_event(&mut gallery.deposit_events, DepositEvent<NFTMeta> { uid: NFT::get_uid(&nft) });
        Collection2::put(sender, receiver, nft);
    }

    /// Withdraw one nft of NFTMeta from `sender`
    public fun withdraw_one<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer): Option<NFT<NFTMeta,NFTBody>> acquires NFTGallery{
        do_withdraw<NFTMeta,NFTBody>(sender, Option::none())
    }

    /// Withdraw nft of NFTMeta and uid from `sender`
    public fun withdraw<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, uid: u64) : Option<NFT<NFTMeta, NFTBody>> acquires NFTGallery{
       do_withdraw(sender, Option::some(uid))
    }

    /// Withdraw nft of NFTMeta and uid from `sender`
    fun do_withdraw<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, uid: Option<u64>) : Option<NFT<NFTMeta, NFTBody>> acquires NFTGallery{
        let sender_addr = Signer::address_of(sender);
        let gallery = borrow_global_mut<NFTGallery<NFTMeta>>(sender_addr);
        let nfts = Collection2::borrow_collection<NFT<NFTMeta, NFTBody>>(sender, sender_addr);
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
                let nft = Collection2::remove<NFT<NFTMeta, NFTBody>>(&mut nfts, i);
                Event::emit_event(&mut gallery.withdraw_events, WithdrawEvent<NFTMeta> { uid: NFT::get_uid(&nft) });
                Option::some(nft)
            }else{
                Option::none()
            }
        };
        Collection2::return_collection(nfts);
        nft
    }

    fun find_by_uid<NFTMeta: copy + store + drop, NFTBody: store>(c: &Collection<NFT<NFTMeta, NFTBody>>, uid: u64): Option<u64>{
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
    public fun count_of<NFTMeta: copy + store + drop, NFTBody: store>(owner: address):u64 {
        Collection2::length_of<NFT<NFTMeta, NFTBody>>(owner)
    }

}
}