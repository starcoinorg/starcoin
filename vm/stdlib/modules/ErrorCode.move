address 0x1 {

module ErrorCode {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    public fun PROLOGUE_ACCOUNT_DOES_NOT_EXIST(): u64 {0}
    public fun PROLOGUE_INVALID_ACCOUNT_AUTH_KEY(): u64 {1}
    public fun PROLOGUE_SEQUENCE_NUMBER_TOO_OLD(): u64 {2}
    public fun PROLOGUE_SEQUENCE_NUMBER_TOO_NEW(): u64 {3}
    public fun PROLOGUE_CANT_PAY_GAS_DEPOSIT(): u64 {4}
    public fun PROLOGUE_TRANSACTION_EXPIRED(): u64 {5}
    public fun PROLOGUE_BAD_CHAIN_ID(): u64 {6}
    public fun PROLOGUE_MODULE_NOT_ALLOWED(): u64 {7}
    public fun PROLOGUE_SCRIPT_NOT_ALLOWED(): u64 {8}

    public fun EINSUFFICIENT_BALANCE(): u64 {10}
    public fun ENOT_GENESIS_ACCOUNT(): u64 {11}
    public fun ENOT_GENESIS(): u64 {12}
    public fun ECONFIG_VALUE_DOES_NOT_EXIST(): u64 {13}
    public fun EINVALID_TIMESTAMP(): u64 {14}
    public fun ECOIN_DEPOSIT_IS_ZERO(): u64 {15}
    public fun EDESTORY_TOKEN_NON_ZERO(): u64 {16}
    public fun EBLOCK_NUMBER_MISMATCH(): u64 {17}
    /// Invalid argument.
    public fun EINVALID_ARGUMENT(): u64 {18}
    /// There code should unreacheable
    public fun EUNREACHABLE(): u64 {19}

    public fun ECODE_BASE(): u64 {100}

}
}