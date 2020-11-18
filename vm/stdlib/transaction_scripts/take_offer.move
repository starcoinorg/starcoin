script {
    use 0x1::Offer;
    use 0x1::Box;

    fun take_offer<Offered: copyable>(
        signer: &signer,
        offer_address: address,
    ) {
        let offered = Offer::redeem<Offered>(signer, offer_address);
        Box::put(signer, offered);
    }

    spec fun take_offer {
        pragma verify = false;
    }
}
