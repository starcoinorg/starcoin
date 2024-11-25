module starcoin_framework::stc_offer {
    use std::error;
    use std::signer;
    use starcoin_framework::timestamp;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    /// A wrapper around value `offered` that can be claimed by the address stored in `for` when after lock time.
    struct Offer<Offered> has key {
        offered: Offered,
        for_address: address,
        time_lock: u64
    }

    /// An offer of the specified type for the account does not match
    const EOFFER_DNE_FOR_ACCOUNT: u64 = 101;

    /// Offer is not unlocked yet.
    const EOFFER_NOT_UNLOCKED: u64 = 102;

    /// Publish a value of type `Offered` under the sender's account. The value can be claimed by
    /// either the `for` address or the transaction sender.
    public fun create<Offered: store>(account: &signer, offered: Offered, for_address: address, lock_period: u64) {
        let time_lock = timestamp::now_seconds() + lock_period;
        //TODO should support multi Offer?
        move_to(account, Offer<Offered> {
            offered,
            for_address,
            time_lock
        });
    }

    spec create {
        use starcoin_framework::timestamp;
        use starcoin_framework::signer;

        // include timestamp::AbortsIfTimestampNotExists;
        aborts_if timestamp::now_seconds() + lock_period > max_u64();
        aborts_if exists<Offer<Offered>>(signer::address_of(account));
    }

    /// Claim the value of type `Offered` published at `offer_address`.
    /// Only succeeds if the sender is the intended recipient stored in `for` or the original
    /// publisher `offer_address`, and now >= time_lock
    /// Also fails if no such value exists.
    public fun redeem<Offered: store>(account: &signer, offer_address: address): Offered acquires Offer {
        let Offer<Offered> { offered, for_address, time_lock } = move_from<Offer<Offered>>(offer_address);
        let sender = signer::address_of(account);
        let now = timestamp::now_seconds();
        assert!(sender == for_address || sender == offer_address, error::invalid_argument(EOFFER_DNE_FOR_ACCOUNT));
        assert!(now >= time_lock, error::invalid_state(EOFFER_NOT_UNLOCKED));
        offered
    }

    spec redeem {
        use starcoin_framework::timestamp;
        use starcoin_framework::signer;

        aborts_if !exists<Offer<Offered>>(offer_address);
        aborts_if
            signer::address_of(account) != global<Offer<Offered>>(offer_address).for_address
                && signer::address_of(account) != offer_address;
        aborts_if timestamp::now_seconds() < global<Offer<Offered>>(offer_address).time_lock;
        // include timestamp::AbortsIfTimestampNotExists;
    }

    /// Returns true if an offer of type `Offered` exists at `offer_address`.
    public fun exists_at<Offered: store>(offer_address: address): bool {
        exists<Offer<Offered>>(offer_address)
    }

    spec exists_at {
        aborts_if false;
    }

    /// Returns the address of the `Offered` type stored at `offer_address`.
    /// Fails if no such `Offer` exists.
    public fun address_of<Offered: store>(offer_address: address): address acquires Offer {
        borrow_global<Offer<Offered>>(offer_address).for_address
    }

    spec address_of {
        aborts_if !exists<Offer<Offered>>(offer_address);
    }

    // /// Take Offer and put to signer's Collection<Offered>.
    // public entry fun take_offer<Offered: store>(
    //     signer: signer,
    //     offer_address: address,
    // ) acquires Offer {
    //     let offered = redeem<Offered>(&signer, offer_address);
    //     Collection2::put(&signer, signer::address_of(&signer), offered);
    // }

    // spec take_offer {
    //     pragma verify = false;
    // }
}