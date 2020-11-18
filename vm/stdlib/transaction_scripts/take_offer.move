script {
    use 0x1::Signer;
    use 0x1::Offer;
    use 0x1::Box;

    fun take_offer<Offered: copyable>(
        signer: &signer,
    ) {
        let offer_address = Signer::address_of(signer);
        let offered = Offer::redeem<Offered>(signer, offer_address);
        Box::put(signer, offered);
    }

    spec fun take_offer {
        pragma verify = false;
    }
}
