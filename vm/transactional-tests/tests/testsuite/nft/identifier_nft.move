//# init -n dev

//# faucet --addr creator

//# faucet --addr bob


//# block --author 0x1

//# publish
//this is a x service's membership nft example
module creator::XMembership {
    use StarcoinFramework::NFT::{Self, MintCapability, BurnCapability, UpdateCapability};
    use StarcoinFramework::IdentifierNFT;
    use StarcoinFramework::Token::{Self, Token};
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Account;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Option;

    struct XMembershipInfo has copy, store, drop, key{
        price_per_millis: u128,
    }

    struct XMembership has copy, store, drop{
        join_time: u64,
        end_time: u64,
    }

    struct XMembershipBody has store{
        fee: Token<STC>,
    }
  
    struct XMembershipMintCapability has key{
        cap: MintCapability<XMembership>,
    }

    struct XMembershipBurnCapability has key{
        cap: BurnCapability<XMembership>,
    }

    struct XMembershipUpdateCapability has key{
        cap: UpdateCapability<XMembership>,
    }

    public fun init(sender: &signer){
        assert!(Signer::address_of(sender) == @creator, 1000);

        NFT::register_v2<XMembership>(sender, NFT::empty_meta());
        move_to(sender, XMembershipInfo{ price_per_millis:2 });
        let cap = NFT::remove_mint_capability<XMembership>(sender);
        move_to(sender, XMembershipMintCapability{ cap});

        let cap = NFT::remove_burn_capability<XMembership>(sender);
        move_to(sender, XMembershipBurnCapability{ cap});

        let cap = NFT::remove_update_capability<XMembership>(sender);
        move_to(sender, XMembershipUpdateCapability{ cap});
    }

    public fun join(sender: &signer, fee: u128) acquires XMembershipMintCapability, XMembershipInfo{
        let token = Account::withdraw<STC>(sender, fee);
        let cap = borrow_global_mut<XMembershipMintCapability>(@creator);
        let metadata = NFT::new_meta_with_image(b"xmembership", b"ipfs:://xxxxxx", b"This is a XMembership nft.");
        let info = borrow_global<XMembershipInfo>(@creator);
        let join_time = Timestamp::now_milliseconds();
        let end_time = join_time + ((Token::value(&token)/info.price_per_millis) as u64);
        let nft = NFT::mint_with_cap_v2<XMembership,XMembershipBody>(@creator, &mut cap.cap, metadata, XMembership{ join_time, end_time}, XMembershipBody{ fee: token});
        IdentifierNFT::grant(&mut cap.cap, sender, nft);
    }

    /// takeout fee when quit
    public fun quit(sender: &signer) acquires XMembershipBurnCapability{
        let cap = borrow_global_mut<XMembershipBurnCapability>(@creator);
        let now = Timestamp::now_milliseconds();
        let addr = Signer::address_of(sender);
        let nft = IdentifierNFT::revoke<XMembership, XMembershipBody>(&mut cap.cap, addr);
        let nft_meta = *NFT::get_type_meta(&nft);
        let XMembershipBody{fee} = NFT::burn_with_cap(&mut cap.cap, nft);
        let real_fee_value = ((now - nft_meta.join_time) as u128);
        let fee_value = Token::value(&fee);
        if (real_fee_value >= fee_value) {
            Account::deposit(@creator, fee);
        }else{
            let real_fee = Token::withdraw(&mut fee, real_fee_value);
            Account::deposit(@creator, real_fee);
            //pay back remain fee.
            Account::deposit_to_self(sender, fee);
        }
    }

    // check memebership in special method.
    public fun do_membership_action(sender: &signer) acquires XMembershipBurnCapability{
        let addr = Signer::address_of(sender);
        assert!(IdentifierNFT::is_owns<XMembership, XMembershipBody>(addr), 1001);
        let nft_info = Option::destroy_some(IdentifierNFT::get_nft_info<XMembership, XMembershipBody>(addr));
        let now = Timestamp::now_milliseconds();
        let (_id,_creator,_metadata,membership) = NFT::unpack_info(nft_info);
        if(membership.end_time <= now){
            Self::quit(sender);
        }else{
            //do other membership jobs
        }
    }
}

// check: EXECUTED

//# run --signers creator
script {
    use creator::XMembership;
    fun main(sender: signer) {
        XMembership::init(&sender);
    }
}

// check: EXECUTED

//# run --signers bob
script {
    use creator::XMembership;
    fun main(sender: signer) {
        XMembership::join(&sender, 100000);
    }
}

// check: EXECUTED



//# run --signers bob
script {
    use creator::XMembership;
    fun main(sender: signer) {
        XMembership::do_membership_action(&sender);
    }
}

// check: EXECUTED


//# block --author 0x1

//# run --signers bob
script {
    use creator::XMembership;
    fun main(sender: signer) {
        XMembership::quit(&sender);
    }
}

// check: EXECUTED
