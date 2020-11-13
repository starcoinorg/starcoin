address 0x1 {
module Offer {
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::Errors;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    // A wrapper around value `offered` that can be claimed by the address stored in `for` when after lock time.
    resource struct Offer<Offered> { offered: Offered, for: address, time_lock: u64 }

    // An offer of the specified type for the account does not match
    const EOFFER_DNE_FOR_ACCOUNT: u64 = 101;

    // Offer is not unlocked yet.
    const EOFFER_NOT_UNLOCKED: u64 = 102;

    // Publish a value of type `Offered` under the sender's account. The value can be claimed by
    // either the `for` address or the transaction sender.
    public fun create<Offered>(account: &signer, offered: Offered, for: address, lock_period: u64) {
        let time_lock = Timestamp::now_seconds() + lock_period;
        //TODO should support multi Offer?
        move_to(account, Offer<Offered> { offered, for, time_lock });
    }

    spec fun create {
        include Timestamp::AbortsIfTimestampNotExists;
        aborts_if Timestamp::now_seconds() + lock_period > max_u64();
        aborts_if exists<Offer<Offered>>(Signer::address_of(account));
    }

    // Claim the value of type `Offered` published at `offer_address`.
    // Only succeeds if the sender is the intended recipient stored in `for` or the original
    // publisher `offer_address`, and now >= time_lock
    // Also fails if no such value exists.
    public fun redeem<Offered>(account: &signer, offer_address: address): Offered acquires Offer {
        let Offer<Offered> { offered, for, time_lock } = move_from<Offer<Offered>>(offer_address);
        let sender = Signer::address_of(account);
        let now = Timestamp::now_seconds();
        assert(sender == for || sender == offer_address, Errors::invalid_argument(EOFFER_DNE_FOR_ACCOUNT));
        assert(now >= time_lock, Errors::not_published(EOFFER_NOT_UNLOCKED));
        offered
    }

    spec fun redeem {
        aborts_if !exists<Offer<Offered>>(offer_address);
        aborts_if Signer::address_of(account) != global<Offer<Offered>>(offer_address).for && Signer::address_of(account) != offer_address;
        aborts_if Timestamp::now_seconds() < global<Offer<Offered>>(offer_address).time_lock;
        include Timestamp::AbortsIfTimestampNotExists;
    }

    // Returns true if an offer of type `Offered` exists at `offer_address`.
    public fun exists_at<Offered>(offer_address: address): bool {
        exists<Offer<Offered>>(offer_address)
    }

    spec fun exists_at {aborts_if false;}

    // Returns the address of the `Offered` type stored at `offer_address.
    // Fails if no such `Offer` exists.
    public fun address_of<Offered>(offer_address: address): address acquires Offer {
        borrow_global<Offer<Offered>>(offer_address).for
    }

    spec fun address_of {aborts_if !exists<Offer<Offered>>(offer_address);}
}
}