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
        id: u64,
        creator: address,
        base_meta: Metadata,
        type_meta: NFTMeta,
    }

    /// The info of NFT type
    struct NFTTypeInfo<NFTMeta: copy + store + drop, NFTTypeInfoExt: copy + store + drop> has key, store {
        counter: u64,
        meta: Metadata,
        info: NFTTypeInfoExt,
        mint_events: Event::EventHandle<MintEvent<NFTMeta>>,
    }

    fun new_nft_type_info<NFTMeta: copy + store + drop, NFTTypeInfoExt: copy + store + drop>(sender: &signer, info: NFTTypeInfoExt, meta: Metadata): NFTTypeInfo<NFTMeta, NFTTypeInfoExt> {
        NFTTypeInfo<NFTMeta, NFTTypeInfoExt> {
            counter: 0,
            info,
            meta,
            mint_events: Event::new_event_handle<MintEvent<NFTMeta>>(sender),
        }
    }

    public fun nft_type_info_ex_info<NFTMeta: copy + store + drop, NFTTypeInfoExt: copy + store + drop>(): NFTTypeInfoExt acquires NFTTypeInfo {
        let info = borrow_global_mut<NFTTypeInfo<NFTMeta, NFTTypeInfoExt>>(CoreAddresses::GENESIS_ADDRESS());
        *&info.info
    }

    public fun nft_type_info_counter<NFTMeta: copy + store + drop, NFTTypeInfoExt: copy + store + drop>(): u64 acquires NFTTypeInfo {
        let info = borrow_global_mut<NFTTypeInfo<NFTMeta, NFTTypeInfoExt>>(CoreAddresses::GENESIS_ADDRESS());
        *&info.counter
    }

    struct GenesisSignerCapability has key {
        cap: Account::SignerCapability,
    }
    /// The capability to mint the nft.
    struct MintCapability<NFTMeta: store> has key, store {}
    /// The Capability to burn the nft.
    struct BurnCapability<NFTMeta: store> has key, store {}
    /// The Capability to update the nft metadata.
    struct UpdateCapability<NFTMeta: store> has key, store {}

    struct Metadata has copy, store, drop {
        /// NFT name's utf8 bytes.
        name: vector<u8>,
        /// Image link, such as ipfs://xxxx
        image: vector<u8>,
        /// Image bytes data, image or image_data can not empty for both.
        image_data: vector<u8>,
        /// NFT description utf8 bytes.
        description: vector<u8>,
    }

    public fun empty_meta(): Metadata {
        Metadata {
            name: Vector::empty(),
            image: Vector::empty(),
            image_data: Vector::empty(),
            description: Vector::empty(),
        }
    }

    public fun new_meta(name: vector<u8>, description: vector<u8>): Metadata {
        Metadata {
            name,
            image: Vector::empty(),
            image_data: Vector::empty(),
            description,
        }
    }

    public fun new_meta_with_image(name: vector<u8>, image: vector<u8>, description: vector<u8>): Metadata {
        assert(!Vector::is_empty(&name), Errors::invalid_argument(ERR_CANOT_EMPTY));
        assert(!Vector::is_empty(&image), Errors::invalid_argument(ERR_CANOT_EMPTY));
        Metadata {
            name,
            image,
            image_data: Vector::empty(),
            description,
        }
    }

    public fun new_meta_with_image_data(name: vector<u8>, image_data: vector<u8>, description: vector<u8>): Metadata {
        assert(!Vector::is_empty(&name), Errors::invalid_argument(ERR_CANOT_EMPTY));
        assert(!Vector::is_empty(&image_data), Errors::invalid_argument(ERR_CANOT_EMPTY));
        Metadata {
            name,
            image: Vector::empty(),
            image_data,
            description,
        }
    }

    public fun meta_name(metadata: &Metadata): vector<u8> {
        *&metadata.name
    }

    public fun meta_image(metadata: &Metadata): vector<u8> {
        *&metadata.image
    }

    public fun meta_image_data(metadata: &Metadata): vector<u8> {
        *&metadata.image_data
    }

    public fun meta_description(metadata: &Metadata): vector<u8> {
        *&metadata.description
    }

    struct NFT<NFTMeta: copy + store + drop, NFTBody> has store {
        /// The creator of NFT
        creator: address,
        /// The unique id of NFT under NFTMeta type
        id: u64,
        /// The metadata of NFT
        base_meta: Metadata,
        /// The extension metadata of NFT
        type_meta: NFTMeta,
        /// The body of NFT, NFT is a box for NFTBody
        body: NFTBody,
    }

    /// The information of NFT instance return by get_nft_info
    struct NFTInfo<NFTMeta: copy + store + drop> has copy, store, drop {
        id: u64,
        creator: address,
        base_meta: Metadata,
        type_meta: NFTMeta,
    }

    public fun get_info<NFTMeta: copy + store + drop, NFTBody: store>(nft: &NFT<NFTMeta, NFTBody>): NFTInfo<NFTMeta> {
        return NFTInfo<NFTMeta> { id: nft.id, creator: nft.creator, base_meta: *&nft.base_meta, type_meta: *&nft.type_meta }
    }

    public fun unpack_info<NFTMeta: copy + store + drop>(nft_info: NFTInfo<NFTMeta>): (u64, address, Metadata, NFTMeta) {
        let NFTInfo<NFTMeta> { id, creator, base_meta, type_meta } = nft_info;
        (id, creator, base_meta, type_meta)
    }

    public fun get_id<NFTMeta: copy + store + drop, NFTBody: store>(nft: &NFT<NFTMeta, NFTBody>): u64 {
        return nft.id
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
    public fun register<NFTMeta: copy + store + drop, NFTTypeInfoExt: copy + store + drop>(sender: &signer, info: NFTTypeInfoExt, meta: Metadata) acquires GenesisSignerCapability {
        let genesis_cap = borrow_global<GenesisSignerCapability>(CoreAddresses::GENESIS_ADDRESS());
        let genesis_account = Account::create_signer_with_cap(&genesis_cap.cap);
        let type_info = new_nft_type_info(sender, info, meta);
        move_to<NFTTypeInfo<NFTMeta, NFTTypeInfoExt>>(&genesis_account, type_info);
        move_to<MintCapability<NFTMeta>>(sender, MintCapability {});
        move_to<BurnCapability<NFTMeta>>(sender, BurnCapability {});
        move_to<UpdateCapability<NFTMeta>>(sender, UpdateCapability {});
    }

    /// Add MintCapability to `sender`
    public fun add_mint_capability<NFTMeta: copy + store + drop>(sender: &signer, cap: MintCapability<NFTMeta>) {
        move_to(sender, cap);
    }

    /// Remove the MintCapability<NFTMeta> from `sender`
    public fun remove_mint_capability<NFTMeta: copy + store + drop>(sender: &signer): MintCapability<NFTMeta> acquires MintCapability {
        let addr = Signer::address_of(sender);
        assert(exists<MintCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        move_from<MintCapability<NFTMeta>>(addr)
    }

    ///Destroy the MintCapability<NFTMeta>
    public fun destroy_mint_capability<NFTMeta: copy + store + drop>(cap: MintCapability<NFTMeta>) {
        let MintCapability {} = cap;
    }

    /// Mint nft with MintCapability<NFTTYpe>, `creator` will been the NFT's creator.
    public fun mint_with_cap<NFTMeta: copy + store + drop, NFTBody: store, NFTTypeInfoExt: copy + store + drop>(creator: address, _cap: &mut MintCapability<NFTMeta>, base_meta: Metadata, type_meta: NFTMeta, body: NFTBody): NFT<NFTMeta, NFTBody> acquires NFTTypeInfo {
        let nft_type_info = borrow_global_mut<NFTTypeInfo<NFTMeta, NFTTypeInfoExt>>(CoreAddresses::GENESIS_ADDRESS());
        nft_type_info.counter = nft_type_info.counter + 1;
        let id = nft_type_info.counter;
        let nft = NFT<NFTMeta, NFTBody> {
            id: id,
            creator,
            base_meta: copy base_meta,
            type_meta: copy type_meta,
            body,
        };
        Event::emit_event(&mut nft_type_info.mint_events, MintEvent<NFTMeta> {
            id,
            creator,
            base_meta,
            type_meta,
        });
        return nft
    }

    /// Mint nft, the `sender` must have MintCapability<NFTMeta>
    public fun mint<NFTMeta: copy + store + drop, NFTBody: store, NFTTypeInfoExt: copy + store + drop>(sender: &signer, base_meta: Metadata, type_meta: NFTMeta, body: NFTBody): NFT<NFTMeta, NFTBody> acquires NFTTypeInfo, MintCapability {
        let addr = Signer::address_of(sender);
        assert(exists<MintCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_MINT_CAPABILITY));
        let cap = borrow_global_mut<MintCapability<NFTMeta>>(addr);
        mint_with_cap<NFTMeta, NFTBody, NFTTypeInfoExt>(addr, cap, base_meta, type_meta, body)
    }

    /// Add BurnCapability<NFTMeta> to `sender`
    public fun add_burn_capability<NFTMeta: copy + store + drop>(sender: &signer, cap: BurnCapability<NFTMeta>) {
        move_to(sender, cap);
    }

    /// Remove the BurnCapability<NFTMeta> from `sender`
    public fun remove_burn_capability<NFTMeta: copy + store + drop>(sender: &signer): BurnCapability<NFTMeta> acquires BurnCapability {
        let addr = Signer::address_of(sender);
        assert(exists<BurnCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_BURN_CAPABILITY));
        move_from<BurnCapability<NFTMeta>>(addr)
    }

    ///Destroy the BurnCapability<NFTMeta>
    public fun destroy_burn_capability<NFTMeta: copy + store + drop>(cap: BurnCapability<NFTMeta>) {
        let BurnCapability {} = cap;
    }

    /// Burn nft with BurnCapability<NFTMeta>
    public fun burn_with_cap<NFTMeta: copy + store + drop, NFTBody: store>(_cap: &mut BurnCapability<NFTMeta>, nft: NFT<NFTMeta, NFTBody>): NFTBody {
        let NFT { creator: _, id: _, base_meta: _, type_meta: _, body } = nft;
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
    public fun add_update_capability<NFTMeta: copy + store + drop>(sender: &signer, cap: UpdateCapability<NFTMeta>) {
        move_to(sender, cap);
    }

    /// Remove the BurnCapability<NFTMeta> from `sender`
    public fun remove_update_capability<NFTMeta: copy + store + drop>(sender: &signer): UpdateCapability<NFTMeta> acquires UpdateCapability {
        let addr = Signer::address_of(sender);
        assert(exists<UpdateCapability<NFTMeta>>(addr), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        move_from<UpdateCapability<NFTMeta>>(addr)
    }

    ///Destroy the UpdateCapability<NFTMeta>
    public fun destroy_update_capability<NFTMeta: copy + store + drop>(cap: UpdateCapability<NFTMeta>) {
        let UpdateCapability {} = cap;
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
    public fun borrow_body_mut_with_cap<NFTMeta: copy + store + drop, NFTBody: store>(_cap: &mut UpdateCapability<NFTMeta>, nft: &mut NFT<NFTMeta, NFTBody>): &mut NFTBody {
        &mut nft.body
    }
}

/// IdentifierNFT using NFT as identifier for an on chain account
/// The NFT can not been transfer by owner.
module IdentifierNFT {
    use 0x1::Option::{Self, Option};
    use 0x1::NFT::{Self, NFT, MintCapability, BurnCapability};
    use 0x1::Signer;
    use 0x1::Errors;

    const ERR_NFT_EXISTS: u64 = 101;
    const ERR_NFT_NOT_EXISTS: u64 = 102;
    const ERR_NFT_NOT_ACCEPT: u64 = 103;

    struct IdentifierNFT<NFTMeta: copy + store + drop, NFTBody: store> has key {
        nft: Option<NFT<NFTMeta, NFTBody>>,
    }

    /// Check the `owner` is prepared with IdentifierNFT for accept the NFT<NFTMeta, NFTBody>
    public fun is_accept<NFTMeta: copy + store + drop, NFTBody: store>(owner: address): bool {
        exists<IdentifierNFT<NFTMeta, NFTBody>>(owner)
    }

    /// Accept NFT<NFTMet, NFTBody>, prepare an empty IdentifierNFT for `sender`
    public fun accept<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer) {
        let addr = Signer::address_of(sender);
        if (!is_accept<NFTMeta, NFTBody>(addr)) {
            move_to(sender, IdentifierNFT<NFTMeta, NFTBody> {
                nft: Option::none(),
            });
        }
    }

    /// Destroy the empty IdentifierNFT
    public fun destroy_empty<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer) acquires IdentifierNFT {
        let addr = Signer::address_of(sender);
        if (exists<IdentifierNFT<NFTMeta, NFTBody>>(addr)) {
            let id_nft = move_from<IdentifierNFT<NFTMeta, NFTBody>>(addr);
            assert(Option::is_none(&id_nft.nft), Errors::already_published(ERR_NFT_EXISTS));
            let IdentifierNFT { nft } = id_nft;
            Option::destroy_none(nft);
        }
    }

    /// Grant nft as IdentifierNFT to `sender` with MintCapability<NFTMeta>, sender will auto accept the NFT.
    public fun grant<NFTMeta: copy + store + drop, NFTBody: store>(cap: &mut MintCapability<NFTMeta>, sender: &signer, nft: NFT<NFTMeta, NFTBody>) acquires IdentifierNFT {
        Self::accept<NFTMeta, NFTBody>(sender);
        Self::grant_to<NFTMeta, NFTBody>(cap, Signer::address_of(sender), nft);
    }

    /// Grant  nft as IdentifierNFT to `receiver` with MintCapability<NFTMeta>, the receiver should accept the NFT first.
    public fun grant_to<NFTMeta: copy + store + drop, NFTBody: store>(_cap: &mut MintCapability<NFTMeta>, receiver: address, nft: NFT<NFTMeta, NFTBody>) acquires IdentifierNFT {
        assert(exists<IdentifierNFT<NFTMeta, NFTBody>>(receiver), Errors::not_published(ERR_NFT_NOT_ACCEPT));
        let id_nft = borrow_global_mut<IdentifierNFT<NFTMeta, NFTBody>>(receiver);
        assert(Option::is_none(&id_nft.nft), Errors::already_published(ERR_NFT_EXISTS));
        Option::fill(&mut id_nft.nft, nft);
    }

    /// Revoke the NFT<NFTMeta, NFTBody> from owner.
    public fun revoke<NFTMeta: copy + store + drop, NFTBody: store>(_cap: &mut BurnCapability<NFTMeta>, owner: address): NFT<NFTMeta, NFTBody>  acquires IdentifierNFT {
        assert(exists<IdentifierNFT<NFTMeta, NFTBody>>(owner), Errors::not_published(ERR_NFT_NOT_EXISTS));
        let id_nft = move_from<IdentifierNFT<NFTMeta, NFTBody>>(owner);
        assert(Option::is_some(&id_nft.nft), Errors::not_published(ERR_NFT_NOT_EXISTS));
        let IdentifierNFT { nft } = id_nft;
        Option::destroy_some(nft)
    }

    /// Check `owner` is owns the IdentifierNFT<NFTMeta, NFTBody>
    public fun is_owns<NFTMeta: copy + store + drop, NFTBody: store>(owner: address): bool acquires IdentifierNFT {
        if (!exists<IdentifierNFT<NFTMeta, NFTBody>>(owner)) {
            return false
        };
        let id_nft = borrow_global<IdentifierNFT<NFTMeta, NFTBody>>(owner);
        Option::is_some(&id_nft.nft)
    }

    public fun get_nft_info<NFTMeta: copy + store + drop, NFTBody: store>(owner: address): Option<NFT::NFTInfo<NFTMeta>> acquires IdentifierNFT {
        if (!exists<IdentifierNFT<NFTMeta, NFTBody>>(owner)) {
            return Option::none<NFT::NFTInfo<NFTMeta>>()
        };
        let id_nft = borrow_global<IdentifierNFT<NFTMeta, NFTBody>>(owner);
        let info = if (Option::is_some(&id_nft.nft)) {
            let nft = Option::borrow(&id_nft.nft);
            Option::some(NFT::get_info(nft))
        } else {
            Option::none<NFT::NFTInfo<NFTMeta>>()
        };
        info
    }
}

module IdentifierNFTScripts {
    use 0x1::IdentifierNFT;

    /// Init IdentifierNFT for accept NFT<NFTMeta, NFTBody> as Identifier.
    public(script) fun accept<NFTMeta: copy + store + drop, NFTBody: store>(sender: signer) {
        IdentifierNFT::accept<NFTMeta, NFTBody>(&sender);
    }
    /// Destroy empty IdentifierNFT
    public(script) fun destroy_empty<NFTMeta: copy + store + drop, NFTBody: store>(sender: signer) {
        IdentifierNFT::destroy_empty<NFTMeta, NFTBody>(&sender);
    }
}

/// NFTGallery is user collection of NFT.
module NFTGallery {
    use 0x1::Signer;
    use 0x1::NFT::{Self, NFT};
    use 0x1::Option::{Self, Option};
    use 0x1::Event;
    use 0x1::Errors;
    use 0x1::Vector;

    const ERR_NFT_NOT_EXISTS: u64 = 101;

    struct WithdrawEvent<NFTMeta: copy + store + drop> has drop, store {
        owner: address,
        id: u64,
    }

    struct DepositEvent<NFTMeta: copy + store + drop> has drop, store {
        owner: address,
        id: u64,
    }

    struct NFTGallery<NFTMeta: copy + store + drop, NFTBody: store> has key, store {
        withdraw_events: Event::EventHandle<WithdrawEvent<NFTMeta>>,
        deposit_events: Event::EventHandle<DepositEvent<NFTMeta>>,
        items: vector<NFT<NFTMeta, NFTBody>>,
    }

    /// Check the `owner` is prepared with NFTGallery for accept the NFT<NFTMeta, NFTBody>
    public fun is_accept<NFTMeta: copy + store + drop, NFTBody: store>(owner: address): bool {
        exists<NFTGallery<NFTMeta, NFTBody>>(owner)
    }

    /// Init a NFTGallery to accept NFT<NFTMeta, NFTBody> for `sender`
    public fun accept<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer) {
        let sender_addr = Signer::address_of(sender);
        if (!is_accept<NFTMeta, NFTBody>(sender_addr)) {
            let gallery = NFTGallery {
                withdraw_events: Event::new_event_handle<WithdrawEvent<NFTMeta>>(sender),
                deposit_events: Event::new_event_handle<DepositEvent<NFTMeta>>(sender),
                items: Vector::empty<NFT<NFTMeta, NFTBody>>(),
            };
            move_to(sender, gallery);
        }
    }

    /// Transfer NFT from `sender` to `receiver`
    public fun transfer<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, id: u64, receiver: address) acquires NFTGallery {
        let nft = withdraw<NFTMeta, NFTBody>(sender, id);
        assert(Option::is_some(&nft), Errors::not_published(ERR_NFT_NOT_EXISTS));
        let nft = Option::destroy_some(nft);
        deposit_to(receiver, nft)
    }

    /// Get the NFT info by the NFT id.
    public fun get_nft_info_by_id<NFTMeta: copy + store + drop, NFTBody: store>(owner: address, id: u64): Option<NFT::NFTInfo<NFTMeta>> acquires NFTGallery {
        let gallery = borrow_global_mut<NFTGallery<NFTMeta, NFTBody>>(owner);
        let idx = find_by_id<NFTMeta, NFTBody>(&gallery.items, id);

        let info = if (Option::is_some(&idx)) {
            let i = Option::extract(&mut idx);
            let nft = Vector::borrow<NFT<NFTMeta, NFTBody>>(&gallery.items, i);
            Option::some(NFT::get_info(nft))
        } else {
            Option::none<NFT::NFTInfo<NFTMeta>>()
        };
        return info
    }

    /// Get the NFT info by the NFT idx in NFTGallery
    public fun get_nft_info_by_idx<NFTMeta: copy + store + drop, NFTBody: store>(owner: address, idx: u64): NFT::NFTInfo<NFTMeta> acquires NFTGallery {
        let gallery = borrow_global_mut<NFTGallery<NFTMeta, NFTBody>>(owner);
        let nft = Vector::borrow<NFT<NFTMeta, NFTBody>>(&gallery.items, idx);
        NFT::get_info(nft)
    }

    /// Get the all NFT info
    public fun get_nft_infos<NFTMeta: copy + store + drop, NFTBody: store>(owner: address): vector<NFT::NFTInfo<NFTMeta>> acquires NFTGallery {
        let gallery = borrow_global_mut<NFTGallery<NFTMeta, NFTBody>>(owner);
        let infos = Vector::empty();
        let len = Vector::length(&gallery.items);
        let idx = 0;
        while (len > idx) {
            let nft = Vector::borrow<NFT<NFTMeta, NFTBody>>(&gallery.items, idx);
            Vector::push_back(&mut infos, NFT::get_info(nft));
            idx = idx + 1;
        };
        infos
    }

    /// Deposit nft to `sender` NFTGallery
    public fun deposit<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, nft: NFT<NFTMeta, NFTBody>) acquires NFTGallery {
        let sender_addr = Signer::address_of(sender);
        deposit_to(sender_addr, nft)
    }

    /// Deposit nft to `receiver` NFTGallery
    public fun deposit_to<NFTMeta: copy + store + drop, NFTBody: store>(receiver: address, nft: NFT<NFTMeta, NFTBody>) acquires NFTGallery {
        let gallery = borrow_global_mut<NFTGallery<NFTMeta, NFTBody>>(receiver);
        Event::emit_event(&mut gallery.deposit_events, DepositEvent<NFTMeta> { id: NFT::get_id(&nft), owner: receiver });
        Vector::push_back(&mut gallery.items, nft);
    }

    /// Withdraw one nft of NFTMeta from `sender`, caller should ensure at least one NFT in the Gallery.
    public fun withdraw_one<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer): NFT<NFTMeta, NFTBody> acquires NFTGallery {
        let nft = do_withdraw<NFTMeta, NFTBody>(sender, Option::none());
        Option::destroy_some(nft)
    }

    /// Withdraw nft of NFTMeta and id from `sender`
    public fun withdraw<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, id: u64): Option<NFT<NFTMeta, NFTBody>> acquires NFTGallery {
        do_withdraw(sender, Option::some(id))
    }

    /// Withdraw nft of NFTMeta and id from `sender`
    fun do_withdraw<NFTMeta: copy + store + drop, NFTBody: store>(sender: &signer, id: Option<u64>): Option<NFT<NFTMeta, NFTBody>> acquires NFTGallery {
        let sender_addr = Signer::address_of(sender);
        let gallery = borrow_global_mut<NFTGallery<NFTMeta, NFTBody>>(sender_addr);
        let len = Vector::length(&gallery.items);
        let nft = if (len == 0) {
            Option::none()
        }else {
            let idx = if (Option::is_some(&id)) {
                let id = Option::extract(&mut id);
                find_by_id(&gallery.items, id)
            }else {
                //default withdraw the last nft.
                Option::some(len - 1)
            };

            if (Option::is_some(&idx)) {
                let i = Option::extract(&mut idx);
                let nft = Vector::remove<NFT<NFTMeta, NFTBody>>(&mut gallery.items, i);
                Event::emit_event(&mut gallery.withdraw_events, WithdrawEvent<NFTMeta> { id: NFT::get_id(&nft), owner: sender_addr });
                Option::some(nft)
            }else {
                Option::none()
            }
        };
        nft
    }

    fun find_by_id<NFTMeta: copy + store + drop, NFTBody: store>(c: &vector<NFT<NFTMeta, NFTBody>>, id: u64): Option<u64> {
        let len = Vector::length(c);
        if (len == 0) {
            return Option::none()
        };
        let idx = len - 1;
        loop {
            let nft = Vector::borrow(c, idx);
            if (NFT::get_id(nft) == id) {
                return Option::some(idx)
            };
            if (idx == 0) {
                return Option::none()
            };
            idx = idx - 1;
        }
    }

    /// Count all NFTs assigned to an owner
    public fun count_of<NFTMeta: copy + store + drop, NFTBody: store>(owner: address): u64 acquires NFTGallery {
        let gallery = borrow_global_mut<NFTGallery<NFTMeta, NFTBody>>(owner);
        Vector::length(&gallery.items)
    }
}

module NFTGalleryScripts {
    use 0x1::NFTGallery;

    /// Init a  NFTGallery for accept NFT<NFTMeta, NFTBody>
    public(script) fun accept<NFTMeta: copy + store + drop, NFTBody: store>(sender: signer) {
        NFTGallery::accept<NFTMeta, NFTBody>(&sender);
    }

    /// Transfer NFT<NFTMeta, NFTBody> with `id` from `sender` to `receiver`
    public(script) fun transfer<NFTMeta: copy + store + drop, NFTBody: store>(sender: signer, id: u64, receiver: address) {
        NFTGallery::transfer<NFTMeta, NFTBody>(&sender, id, receiver);
    }
}
}