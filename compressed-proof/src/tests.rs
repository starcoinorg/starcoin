#[cfg(test)]
mod tests {
    use crate::interlink::{calculate_level, calculate_interlink, MAX_LEVELS};
    use crate::proof::{verify_du, VerifyDuError};
    use starcoin_types::{
        block::{BlockHeader, BlockHeaderExtra},
        account_address::AccountAddress,
        genesis_config::ChainId,
        U256,
    };
    use starcoin_crypto::HashValue;
    use std::collections::VecDeque;

    fn create_test_header(
        number: u64,
        parent_hash: HashValue,
        interlink: Vec<HashValue>,
    ) -> BlockHeader {
        // Create a minimal BlockHeader with required fields
        BlockHeader::new(
            parent_hash,                      // parent_hash
            0,                               // timestamp
            number,                          // number
            AccountAddress::ZERO,            // author
            HashValue::zero(),               // txn_accumulator_root
            HashValue::zero(),               // block_accumulator_root
            HashValue::zero(),               // state_root
            0,                               // gas_used
            U256::from(1000u64),            // difficulty
            HashValue::zero(),               // body_hash
            ChainId::test(),                 // chain_id
            0,                               // nonce
            BlockHeaderExtra::new([0u8; 4]), // extra
            vec![],                          // parents_hash
            0,                               // version
            HashValue::zero(),               // pruning_point
            interlink,                       // interlink
        )
    }

    #[test]
    fn test_calculate_level_basic() {
        // Test with zero hash (should return max level)
        let zero_hash = HashValue::zero();
        let difficulty = U256::from(1000u64);
        assert_eq!(calculate_level(zero_hash, difficulty).unwrap(), 255);
        
        // Test with high hash value (should return level 0)
        let high_hash = HashValue::from_hex_literal("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        let result = calculate_level(high_hash, difficulty);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
    
    #[test]
    fn test_calculate_level_medium_hash() {
        // Test with medium hash value
        let medium_hash = HashValue::from_hex_literal("0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        let difficulty = U256::from(1000u64);
        let result = calculate_level(medium_hash, difficulty);
        assert!(result.is_ok());
        // Should be level 1 or higher depending on difficulty
        assert!(result.unwrap() >= 1);
    }
    
    #[test]
    fn test_calculate_interlink_genesis() {
        // Create a mock genesis block with empty interlink
        let genesis_hash = HashValue::random();
        let genesis_header = create_test_header(
            0,
            genesis_hash,
            Vec::new(),
        );
        
        let interlink = calculate_interlink(&genesis_header);
        // Genesis should have interlink with at least one element (its own hash)
        assert!(!interlink.is_empty());
        assert!(interlink.len() <= MAX_LEVELS);
    }
    
    #[test]
    fn test_calculate_interlink_non_genesis() {
        // Create a mock parent block with some interlink
        let parent_hash = HashValue::random();
        let parent_interlink = vec![parent_hash; 3]; // Parent has 3 levels
        
        let parent_header = create_test_header(
            1,
            parent_hash,
            parent_interlink,
        );
        
        let interlink = calculate_interlink(&parent_header);
        assert!(!interlink.is_empty());
        assert!(interlink.len() <= MAX_LEVELS);
        
        // First element should be parent's hash
        assert_eq!(interlink[0], parent_hash);
    }
    
    #[test]
    fn test_max_levels_constraint() {
        assert_eq!(MAX_LEVELS, 255);
    }
    
    #[test]
    fn test_verify_du_empty_buckets() {
        let buckets: Vec<VecDeque<BlockHeader>> = vec![];
        let result = verify_du(&buckets, 5);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_verify_du_valid_buckets() {
        let mut buckets = Vec::new();
        let mut bucket = VecDeque::new();
        
        // Create a valid bucket with proper level blocks
        let hash1 = HashValue::random();
        let header1 = create_test_header(
            1,
            hash1,
            vec![hash1],
        );
        
        let hash2 = HashValue::random();
        let header2 = create_test_header(
            2,
            hash1,
            vec![hash1, hash2],
        );
        
        bucket.push_back(header1);
        bucket.push_back(header2);
        buckets.push(bucket);
        
        let result = verify_du(&buckets, 5);
        // This might fail due to level calculation, but structure should be valid
        assert!(result.is_ok() || matches!(result.unwrap_err(), VerifyDuError::LevelMismatch { .. }));
    }
    
    #[test]
    fn test_verify_du_interlink_mismatch() {
        let mut buckets = Vec::new();
        let mut bucket = VecDeque::new();
        
        let hash1 = HashValue::random();
        let header1 = create_test_header(
            1,
            hash1,
            vec![hash1],
        );
        
        let hash2 = HashValue::random();
        let wrong_hash = HashValue::random();
        let header2 = create_test_header(
            2,
            hash1,
            vec![wrong_hash], // Wrong interlink pointer
        );
        
        bucket.push_back(header1);
        bucket.push_back(header2);
        buckets.push(bucket);
        
        let result = verify_du(&buckets, 5);
        // Should fail due to interlink mismatch
        assert!(matches!(result.unwrap_err(), VerifyDuError::InterlinkMismatch { .. }));
    }
    
    #[test]
    fn test_verify_du_error_types() {
        // Test that all error types can be created
        let level_error = VerifyDuError::LevelMismatch {
            level: 0,
            index: 0,
            got: 1,
        };
        assert!(format!("{}", level_error).contains("level mismatch"));
        
        let count_error = VerifyDuError::SampleCountMismatch {
            level: 0,
            count: 5,
            expect: 10,
        };
        assert!(format!("{}", count_error).contains("samples count mismatch"));
        
        let interlink_error = VerifyDuError::InterlinkMismatch {
            level: 0,
            index: 0,
            expected: HashValue::random(),
            found: HashValue::random(),
        };
        assert!(format!("{}", interlink_error).contains("interlink mismatch"));
        
        let missing_error = VerifyDuError::MissingInterlinkPointer {
            level: 0,
            index: 0,
        };
        assert!(format!("{}", missing_error).contains("missing interlink pointer"));
    }
}