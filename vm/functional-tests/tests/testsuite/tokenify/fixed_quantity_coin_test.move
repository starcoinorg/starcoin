//! account: alice, 0x1
//! account: bob

//! new-transaction
//! sender: alice
script {
    use 0x1::FixedQuantityCoin;
    use 0x1::Signer;
    fun main(signer: &signer) {
        FixedQuantityCoin::initialize(signer);
        assert(
            FixedQuantityCoin::balance(Signer::address_of(signer)) ==
                FixedQuantityCoin::total_supply(),
            100
        );
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob

script {
    use 0x1::FixedQuantityCoin;
    fun main(signer: &signer) {
        FixedQuantityCoin::accept(signer);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice

script {
    use 0x1::FixedQuantityCoin;
    use 0x1::Signer;
    fun main(signer: &signer) {
        let amount = 100000;
        let my_balance = FixedQuantityCoin::balance(Signer::address_of(signer));
        FixedQuantityCoin::transfer_to(signer, {{bob}}, amount);
        assert(FixedQuantityCoin::balance(Signer::address_of(signer)) == my_balance - amount, 100);
        assert(FixedQuantityCoin::balance({{bob}}) == amount, 100);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::FixedQuantityCoin;
    fun main(signer: &signer) {
        FixedQuantityCoin::initialize(signer);
    }
}

// check: CANNOT_WRITE_EXISTING_RESOURCE