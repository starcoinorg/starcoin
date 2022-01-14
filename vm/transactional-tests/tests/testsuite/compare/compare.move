//# init -n dev

//# faucet --addr alice

//# run --signers alice
// Tests for polymorphic comparison in Move
script {
    use StarcoinFramework::Compare;
    use StarcoinFramework::BCS;

    const EQUAL: u8 = 0;
    const LESS_THAN: u8 = 1;
    const GREATER_THAN: u8 = 2;

    fun main() {
        // equality of simple types
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&true), &BCS::to_bytes(&true)) == EQUAL, 8001);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u8), &BCS::to_bytes(&1u8)) == EQUAL, 8002);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1), &BCS::to_bytes(&1)) == EQUAL, 8003);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u128), &BCS::to_bytes(&1u128)) == EQUAL, 8004);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x1), &BCS::to_bytes(&@0x1)) == EQUAL, 8005);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"01")) == EQUAL, 8006);

        // inequality of simple types
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&true), &BCS::to_bytes(&false)) != EQUAL, 8007);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u8), &BCS::to_bytes(&0u8)) != EQUAL, 8008);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1), &BCS::to_bytes(&0)) != EQUAL, 8009);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u128), &BCS::to_bytes(&0u128)) != EQUAL, 8010);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x1), &BCS::to_bytes(&@0x0)) != EQUAL, 8011);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"00")) != EQUAL, 8012);

        // less than for types with a natural ordering exposed via bytecode operations
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&false), &BCS::to_bytes(&true)) == LESS_THAN, 8013);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&0u8), &BCS::to_bytes(&1u8)) == LESS_THAN, 8014);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&0), &BCS::to_bytes(&1)) == LESS_THAN, 8015);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&0u128), &BCS::to_bytes(&1u128)) == LESS_THAN, 8016);

        // less then for types without a natural ordering exposed by bytecode operations
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x0), &BCS::to_bytes(&@0x1)) == LESS_THAN, 8017); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x01), &BCS::to_bytes(&@0x10)) == LESS_THAN, 8018); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x100), &BCS::to_bytes(&@0x001)) == LESS_THAN, 8019); // potentially confusing
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"00"), &BCS::to_bytes(&x"01")) == LESS_THAN, 8020); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"10")) == LESS_THAN, 8021); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"0000"), &BCS::to_bytes(&x"01")) == LESS_THAN, 8022); //
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"0100"), &BCS::to_bytes(&x"0001")) == LESS_THAN, 8023); // potentially confusing

        // greater than for types with a natural ordering exposed by bytecode operations
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&true), &BCS::to_bytes(&false)) == GREATER_THAN, 8024);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u8), &BCS::to_bytes(&0u8)) == GREATER_THAN, 8025);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1), &BCS::to_bytes(&0)) == GREATER_THAN, 8026);
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&1u128), &BCS::to_bytes(&0u128)) == GREATER_THAN, 8027);

        // greater than for types without a natural ordering exposed by by bytecode operations
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x1), &BCS::to_bytes(&@0x0)) == GREATER_THAN, 8028); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x10), &BCS::to_bytes(&@0x01)) == GREATER_THAN, 8029); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&@0x001), &BCS::to_bytes(&@0x100)) == GREATER_THAN, 8030); // potentially confusing
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"00")) == GREATER_THAN, 8031); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"10"), &BCS::to_bytes(&x"01")) == GREATER_THAN, 8032); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"01"), &BCS::to_bytes(&x"0000")) == GREATER_THAN, 8033); // sensible
        assert!(Compare::cmp_bcs_bytes(&BCS::to_bytes(&x"0001"), &BCS::to_bytes(&x"0100")) == GREATER_THAN, 8034); // potentially confusing
}
}
