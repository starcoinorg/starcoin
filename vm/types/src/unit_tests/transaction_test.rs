// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::{Package, RawUserTransaction, SignedUserTransaction, TransactionPayload};
use bcs_ext::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;
use starcoin_crypto::ed25519::{self};

proptest! {
    #[test]
    fn test_sign_raw_transaction(raw_txn in any::<RawUserTransaction>(), keypair in ed25519::keypair_strategy()) {
        let txn = raw_txn.sign(&keypair.private_key, keypair.public_key).unwrap();
        let signed_txn = txn.into_inner();
        assert!(signed_txn.check_signature().is_ok());
    }

    #[test]
    fn transaction_payload_bcs_roundtrip(txn_payload in any::<TransactionPayload>()) {
        assert_canonical_encode_decode(txn_payload);
    }

    #[test]
    fn transaction_package_bcs_roundtrip(txn_package in any::<Package>()) {
        assert_canonical_encode_decode(txn_package);
    }

    #[test]
    fn raw_transaction_bcs_roundtrip(raw_txn in any::<RawUserTransaction>()) {
        assert_canonical_encode_decode(raw_txn);
    }

    #[test]
    fn signed_transaction_bcs_roundtrip(signed_txn in any::<SignedUserTransaction>()) {
        assert_canonical_encode_decode(signed_txn);
    }

}
