//! account: alice, 10000000 0x1::STC::STC

// create txn sender account
//! sender: alice
script {
    use 0x1::STC::STC;
    use 0x1::Account;
    use 0x1::Authenticator;

    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let address = Authenticator::derived_address(auth_key_vec);
        Account::create_account_with_address<STC>(address);
        Account::pay_from<STC>(&account, address, 5000);
    }
}
// check: EXECUTED

// prologue sender is not genesis
//! new-transaction
//! sender: alice
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let seq = Account::sequence_number(txn_sender);
        assert(seq == 0, 1001);
        let balance = Account::balance<STC>(txn_sender);
        assert(balance == 5000, 1001);

        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        Account::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 2818"

// gas is not enough
//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 10000; //EPROLOGUE_CANT_PAY_GAS_DEPOSIT

        Account::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 1031"

// invalid pub key
//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let wrong_txn_public_key = x"c48b687a";
        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        Account::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            wrong_txn_public_key, //EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 263"

// sequence number too new
//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let seq = Account::sequence_number(txn_sender);
        assert(seq == 0, 1005);

        let txn_sequence_number = 1; //EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        Account::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 775"

// successfully executed
//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let seq = Account::sequence_number(txn_sender);
        assert(seq == 0, 1005);

        let txn_sequence_number = 0;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        Account::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );

        // execute the txn...

        let gas_units_remaining = 10;

        Account::txn_epilogue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining
        );
        let seq = Account::sequence_number(txn_sender);
        assert(seq == 1, 1006);
    }
}
// check: EXECUTED

// sequence number too old
//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let seq = Account::sequence_number(txn_sender);
        assert(seq == 1, 1005);

        let txn_sequence_number = 0; //EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;

        Account::txn_prologue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units
        );
    }
}
// check: "Keep(ABORTED { code: 519"

// epilouge insufficient balance
//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let seq = Account::sequence_number(txn_sender);
        assert(seq == 1, 1007);

        let txn_sequence_number = 1;
        let txn_gas_price = 1;
        let txn_max_gas_units = 6000; //EINSUFFICIENT_BALANCE
        let gas_units_remaining = 10;

        Account::txn_epilogue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining
        );
    }
}
// check: "Keep(ABORTED { code: 2568"

// epilogue sender is not genesis
//! new-transaction
//! sender: alice
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Authenticator;
    use 0x1::Vector;
    fun main(account: signer) {
        let txn_public_key = x"c48b687a1dd8265101b33df6ae0b6825234e3f28df9ecb38fb286cf76dae919d";
        let auth_key_vec = Authenticator::ed25519_authentication_key(copy txn_public_key);
        let txn_sender = Authenticator::derived_address(copy auth_key_vec);
        Vector::push_back(&mut txn_public_key, 0u8); //create preimage

        let seq = Account::sequence_number(txn_sender);
        assert(seq == 1, 1007);

        let txn_sequence_number = 1;
        let txn_gas_price = 1;
        let txn_max_gas_units = 1000;
        let gas_units_remaining = 10;

        Account::txn_epilogue<STC>(
            &account,
            txn_sender,
            txn_sequence_number,
            txn_gas_price,
            txn_max_gas_units,
            gas_units_remaining
        );
    }
}
// check: "Keep(ABORTED { code: 2818"