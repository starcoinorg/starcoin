// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_metadata::BlockMetadata;
use proptest::prelude::*;
use scs::test_helpers::assert_canonical_encode_decode;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_block_metadata_canonical_serialization(data in any::<BlockMetadata>()) {
        assert_canonical_encode_decode(data);
    }
}
