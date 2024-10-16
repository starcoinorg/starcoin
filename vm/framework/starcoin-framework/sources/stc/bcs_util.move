module starcoin_framework::bcs_util {

    use std::error;
    use std::option;
    use std::vector;
    use starcoin_std::from_bcs;

    const ERR_INPUT_NOT_LARGE_ENOUGH: u64 = 201;
    const ERR_UNEXPECTED_BOOL_VALUE: u64 = 205;
    const ERR_OVERFLOW_PARSING_ULEB128_ENCODED_UINT32: u64 = 206;
    const ERR_INVALID_ULEB128_NUMBER_UNEXPECTED_ZERO_DIGIT: u64 = 207;
    const INTEGER32_MAX_VALUE: u64 = 2147483647;

    public fun deserialize_option_bytes_vector(
        input: &vector<u8>,
        offset: u64
    ): (vector<option::Option<vector<u8>>>, u64) {
        let (len, new_offset) = deserialize_len(input, offset);
        let i = 0;
        let vec = vector::empty<option::Option<vector<u8>>>();
        while (i < len) {
            let (opt_bs, o) = deserialize_option_bytes(input, new_offset);
            vector::push_back(&mut vec, opt_bs);
            new_offset = o;
            i = i + 1;
        };
        (vec, new_offset)
    }

    spec deserialize_option_bytes_vector {
        pragma verify = false;
    }

    public fun deserialize_bytes_vector(input: &vector<u8>, offset: u64): (vector<vector<u8>>, u64) {
        let (len, new_offset) = deserialize_len(input, offset);
        let i = 0;
        let vec = vector::empty<vector<u8>>();
        while (i < len) {
            let (opt_bs, o) = deserialize_bytes(input, new_offset);
            vector::push_back(&mut vec, opt_bs);
            new_offset = o;
            i = i + 1;
        };
        (vec, new_offset)
    }

    spec deserialize_bytes_vector {
        pragma verify = false;
    }

    #[test] use std::bcs;

    #[test]
    public fun test_deserialize_bytes_array() {
        let hello = b"hello";
        let world = b"world";
        let hello_world = vector::empty<vector<u8>>();
        vector::push_back(&mut hello_world, hello);
        vector::push_back(&mut hello_world, world);
        let bs = bcs::to_bytes<vector<vector<u8>>>(&hello_world);
        let (r, _) = deserialize_bytes_vector(&bs, 0);
        assert!(hello_world == r, 1001);
    }

    public fun deserialize_u64_vector(input: &vector<u8>, offset: u64): (vector<u64>, u64) {
        let (len, new_offset) = deserialize_len(input, offset);
        let i = 0;
        let vec = vector::empty<u64>();
        while (i < len) {
            let (opt_bs, o) = deserialize_u64(input, new_offset);
            vector::push_back(&mut vec, opt_bs);
            new_offset = o;
            i = i + 1;
        };
        (vec, new_offset)
    }

    spec deserialize_u64_vector {
        pragma verify = false;
    }

    public fun deserialize_u128_vector(input: &vector<u8>, offset: u64): (vector<u128>, u64) {
        let (len, new_offset) = deserialize_len(input, offset);
        let i = 0;
        let vec = vector::empty<u128>();
        while (i < len) {
            let (opt_bs, o) = deserialize_u128(input, new_offset);
            vector::push_back(&mut vec, opt_bs);
            new_offset = o;
            i = i + 1;
        };
        (vec, new_offset)
    }

    spec deserialize_u128_vector {
        pragma verify = false;
    }

    #[test]
    public fun test_deserialize_u128_array() {
        let hello: u128 = 1111111;
        let world: u128 = 2222222;
        let hello_world = vector::empty<u128>();
        vector::push_back(&mut hello_world, hello);
        vector::push_back(&mut hello_world, world);
        let bs = bcs::to_bytes<vector<u128>>(&hello_world);
        let (r, _) = deserialize_u128_vector(&bs, 0);
        assert!(hello_world == r, 1002);
    }

    public fun deserialize_option_bytes(input: &vector<u8>, offset: u64): (option::Option<vector<u8>>, u64) {
        let (tag, new_offset) = deserialize_option_tag(input, offset);
        if (!tag) {
            return (option::none<vector<u8>>(), new_offset)
        } else {
            let (bs, new_offset) = deserialize_bytes(input, new_offset);
            return (option::some<vector<u8>>(bs), new_offset)
        }
    }

    spec deserialize_option_bytes {
        pragma verify = false;
    }

    public fun deserialize_address(input: &vector<u8>, offset: u64): (address, u64) {
        let (content, new_offset) = deserialize_16_bytes(input, offset);
        (from_bcs::to_address(content), new_offset)
    }

    spec deserialize_address {
        pragma verify = false;
    }

    #[test]
    fun test_deserialize_address() {
        let addr = @0x18351d311d32201149a4df2a9fc2db8a;
        let bs = bcs::to_bytes<address>(&addr);
        let (r, offset) = deserialize_address(&bs, 0);
        assert!(addr == r, 1003);
        assert!(offset == 16, 1004);
    }

    public fun deserialize_16_bytes(input: &vector<u8>, offset: u64): (vector<u8>, u64) {
        let content = get_n_bytes(input, offset, 16);
        (content, offset + 16)
    }

    spec deserialize_16_bytes {
        pragma verify = false;
    }

    public fun deserialize_bytes(input: &vector<u8>, offset: u64): (vector<u8>, u64) {
        let (len, new_offset) = deserialize_len(input, offset);
        let content = get_n_bytes(input, new_offset, len);
        (content, new_offset + len)
    }

    spec deserialize_bytes {
        pragma verify = false;
    }

    #[test]
    public fun test_deserialize_bytes() {
        let hello = b"hello world";
        let bs = bcs::to_bytes<vector<u8>>(&hello);
        let (r, _) = deserialize_bytes(&bs, 0);
        assert!(hello == r, 1005);
    }

    public fun deserialize_u128(input: &vector<u8>, offset: u64): (u128, u64) {
        let u = get_n_bytes_as_u128(input, offset, 16);
        (u, offset + 16)
    }

    spec deserialize_u128 {
        pragma verify = false;
    }

    #[test]
    fun test_deserialize_u128() {
        let max_int128 = 170141183460469231731687303715884105727;
        let u: u128 = max_int128;
        let bs = bcs::to_bytes<u128>(&u);
        let (r, offset) = deserialize_u128(&bs, 0);
        assert!(u == r, 1006);
        assert!(offset == 16, 1007);
    }


    public fun deserialize_u64(input: &vector<u8>, offset: u64): (u64, u64) {
        let u = get_n_bytes_as_u128(input, offset, 8);
        ((u as u64), offset + 8)
    }

    spec deserialize_u64 {
        pragma verify = false;
    }

    #[test]
    fun test_deserialize_u64() {
        let u: u64 = 12811111111111;
        let bs = bcs::to_bytes<u64>(&u);
        let (r, offset) = deserialize_u64(&bs, 0);
        assert!(u == r, 1008);
        assert!(offset == 8, 1009);
    }

    public fun deserialize_u32(input: &vector<u8>, offset: u64): (u64, u64) {
        let u = get_n_bytes_as_u128(input, offset, 4);
        ((u as u64), offset + 4)
    }

    spec deserialize_u32 {
        pragma verify = false;
    }

    #[test]
    fun test_deserialize_u32() {
        let u: u64 = 1281111;
        let bs = bcs::to_bytes<u64>(&u);
        let (r, offset) = deserialize_u32(&bs, 0);
        _ = r;
        assert!(u == r, 1010);
        assert!(offset == 4, 1011);
    }

    public fun deserialize_u16(input: &vector<u8>, offset: u64): (u64, u64) {
        let u = get_n_bytes_as_u128(input, offset, 2);
        ((u as u64), offset + 2)
    }

    spec deserialize_u16 {
        pragma verify = false;
    }

    public fun deserialize_u8(input: &vector<u8>, offset: u64): (u8, u64) {
        let u = get_byte(input, offset);
        (u, offset + 1)
    }

    spec deserialize_u8 {
        pragma verify = false;
    }

    #[test]
    fun test_deserialize_u8() {
        let u: u8 = 128;
        let bs = bcs::to_bytes<u8>(&u);
        let (r, offset) = deserialize_u8(&bs, 0);
        assert!(u == r, 1012);
        assert!(offset == 1, 1013);
    }

    public fun deserialize_option_tag(input: &vector<u8>, offset: u64): (bool, u64) {
        deserialize_bool(input, offset)
    }

    spec deserialize_option_tag {
        pragma verify = false;
    }

    public fun deserialize_len(input: &vector<u8>, offset: u64): (u64, u64) {
        deserialize_uleb128_as_u32(input, offset)
    }

    spec deserialize_len {
        pragma verify = false;
    }

    public fun deserialize_bool(input: &vector<u8>, offset: u64): (bool, u64) {
        let b = get_byte(input, offset);
        if (b == 1) {
            return (true, offset + 1)
        } else if (b == 0) {
            return (false, offset + 1)
        } else {
            abort ERR_UNEXPECTED_BOOL_VALUE
        }
    }

    spec deserialize_bool {
        pragma verify = false;
    }

    #[test]
    public fun test_deserialize_bool() {
        let t = true;
        let bs = bcs::to_bytes<bool>(&t);
        let (d, _) = deserialize_bool(&bs, 0);
        assert!(d, 1014);

        let f = false;
        bs = bcs::to_bytes<bool>(&f);
        (d, _) = deserialize_bool(&bs, 0);
        assert!(!d, 1015);
    }

    fun get_byte(input: &vector<u8>, offset: u64): u8 {
        assert!(
            ((offset + 1) <= vector::length(input)) && (offset < offset + 1),
            error::invalid_state(ERR_INPUT_NOT_LARGE_ENOUGH)
        );
        *vector::borrow(input, offset)
    }

    spec get_byte {
        pragma verify = false;
    }

    fun get_n_bytes(input: &vector<u8>, offset: u64, n: u64): vector<u8> {
        assert!(
            ((offset + n) <= vector::length(input)) && (offset < offset + n),
            error::invalid_state(ERR_INPUT_NOT_LARGE_ENOUGH)
        );
        let i = 0;
        let content = vector::empty<u8>();
        while (i < n) {
            let b = *vector::borrow(input, offset + i);
            vector::push_back(&mut content, b);
            i = i + 1;
        };
        content
    }

    spec get_n_bytes {
        pragma verify = false;
    }

    fun get_n_bytes_as_u128(input: &vector<u8>, offset: u64, n: u64): u128 {
        assert!(
            ((offset + n) <= vector::length(input)) && (offset < offset + n),
            error::invalid_state(ERR_INPUT_NOT_LARGE_ENOUGH)
        );
        let number: u128 = 0;
        let i = 0;
        while (i < n) {
            let byte = *vector::borrow(input, offset + i);
            let s = (i as u8) * 8;
            number = number + ((byte as u128) << s);
            i = i + 1;
        };
        number
    }

    spec get_n_bytes_as_u128 {
        pragma verify = false;
    }

    public fun deserialize_uleb128_as_u32(input: &vector<u8>, offset: u64): (u64, u64) {
        let value: u64 = 0;
        let shift = 0;
        let new_offset = offset;
        while (shift < 32) {
            let x = get_byte(input, new_offset);
            new_offset = new_offset + 1;
            let digit: u8 = x & 0x7F;
            value = value | (digit as u64) << shift;
            if ((value < 0) || (value > INTEGER32_MAX_VALUE)) {
                abort ERR_OVERFLOW_PARSING_ULEB128_ENCODED_UINT32
            };
            if (digit == x) {
                if (shift > 0 && digit == 0) {
                    abort ERR_INVALID_ULEB128_NUMBER_UNEXPECTED_ZERO_DIGIT
                };
                return (value, new_offset)
            };
            shift = shift + 7
        };
        abort ERR_OVERFLOW_PARSING_ULEB128_ENCODED_UINT32
    }

    spec deserialize_uleb128_as_u32 {
        pragma opaque;
        pragma verify = false;
    }

    #[test]
    public fun test_deserialize_uleb128_as_u32() {
        let i: u64 = 0x7F;
        let bs = serialize_u32_as_uleb128(i);
        let (len, _) = deserialize_uleb128_as_u32(&bs, 0);
        assert!(len == i, 1016);

        let i2: u64 = 0x8F;
        let bs2 = serialize_u32_as_uleb128(i2);
        (len, _) = deserialize_uleb128_as_u32(&bs2, 0);
        assert!(len == i2, 1017);
    }


    #[test]
    public fun test_deserialize_uleb128_as_u32_max_int() {
        let max_int: u64 = 2147483647;

        let bs = serialize_u32_as_uleb128(max_int);
        let (len, _) = deserialize_uleb128_as_u32(&bs, 0);
        assert!(len == max_int, 1018);
    }

    #[test]
    #[expected_failure(abort_code = 206, location= StarcoinFramework::BCS)]
    public fun test_deserialize_uleb128_as_u32_exceeded_max_int() {
        let max_int: u64 = 2147483647;
        let exceeded_max_int: u64 = max_int + 1;

        let bs = serialize_u32_as_uleb128(exceeded_max_int);
        let (_, _) = deserialize_uleb128_as_u32(&bs, 0);
    }


    fun serialize_u32_as_uleb128(value: u64): vector<u8> {
        let output = vector::empty<u8>();
        while ((value >> 7) != 0) {
            vector::push_back(&mut output, (((value & 0x7f) | 0x80) as u8));
            value = value >> 7;
        };
        vector::push_back(&mut output, (value as u8));
        output
    }

    spec serialize_u32_as_uleb128 {
        pragma verify = false;
    }

    // skip Vector<option::Option<vector<u8>>>
    public fun skip_option_bytes_vector(input: &vector<u8>, offset: u64): u64 {
        let (len, new_offset) = deserialize_len(input, offset);
        let i = 0;
        while (i < len) {
            new_offset = skip_option_bytes(input, new_offset);
            i = i + 1;
        };
        new_offset
    }

    spec skip_option_bytes_vector {
        pragma verify = false;
    }

    #[test]
    fun test_skip_option_bytes_vector() {
        let vec = vector::empty<option::Option<vector<u8>>>();
        vector::push_back(&mut vec, option::some(x"01020304"));
        vector::push_back(&mut vec, option::some(x"04030201"));
        let vec = bcs::to_bytes(&vec);
        //vec : [2, 1, 4, 1, 2, 3, 4, 1, 4, 4, 3, 2, 1]
        assert!(skip_option_bytes_vector(&vec, 0) == 13, 2000);
    }

    // skip option::Option<vector<u8>>
    public fun skip_option_bytes(input: &vector<u8>, offset: u64): u64 {
        let (tag, new_offset) = deserialize_option_tag(input, offset);
        if (!tag) {
            new_offset
        } else {
            skip_bytes(input, new_offset)
        }
    }

    spec skip_option_bytes {
        pragma verify = false;
    }

    #[test]
    fun test_skip_none_option_bytes() {
        let op = option::none<vector<u8>>();
        let op = bcs::to_bytes(&op);
        let vec = bcs::to_bytes(&x"01020304");
        vector::append(&mut op, vec);
        // op : [0, 4, 1, 2, 3, 4]
        assert!(skip_option_bytes(&op, 0) == 1, 2007);
    }

    #[test]
    fun test_skip_some_option_bytes() {
        let op = option::some(x"01020304");
        let op = bcs::to_bytes(&op);
        let vec = bcs::to_bytes(&x"01020304");
        vector::append(&mut op, vec);
        // op : [1, 4, 1, 2, 3, 4, 4, 1, 2, 3, 4]
        assert!(skip_option_bytes(&op, 0) == 6, 2008);
    }

    // skip vector<vector<u8>>
    public fun skip_bytes_vector(input: &vector<u8>, offset: u64): u64 {
        let (len, new_offset) = deserialize_len(input, offset);
        let i = 0;
        while (i < len) {
            new_offset = skip_bytes(input, new_offset);
            i = i + 1;
        };
        new_offset
    }

    spec skip_bytes_vector {
        pragma verify = false;
    }

    // skip vector<u8>
    public fun skip_bytes(input: &vector<u8>, offset: u64): u64 {
        let (len, new_offset) = deserialize_len(input, offset);
        new_offset + len
    }

    spec skip_bytes {
        pragma verify = false;
    }

    #[test]
    fun test_skip_bytes() {
        let vec = bcs::to_bytes(&x"01020304");
        let u_64 = bcs::to_bytes(&10);
        vector::append(&mut vec, u_64);
        // vec : [4, 1, 2, 3, 4, 10, 0, 0, 0, 0, 0, 0, 0]
        assert!(skip_bytes(&vec, 0) == 5, 2001);
    }

    // skip some bytes
    public fun skip_n_bytes(input: &vector<u8>, offset: u64, n: u64): u64 {
        can_skip(input, offset, n);
        offset + n
    }

    spec skip_n_bytes {
        pragma verify = false;
    }

    #[test]
    fun test_skip_n_bytes() {
        let vec = bcs::to_bytes(&x"01020304");
        let u_64 = bcs::to_bytes(&10);
        vector::append(&mut vec, u_64);
        // vec : [4, 1, 2, 3, 4, 10, 0, 0, 0, 0, 0, 0, 0]
        assert!(skip_n_bytes(&vec, 0, 1) == 1, 2002);
    }

    // skip vector<u64>
    public fun skip_u64_vector(input: &vector<u8>, offset: u64): u64 {
        let (len, new_offset) = deserialize_len(input, offset);
        can_skip(input, new_offset, len * 8);
        new_offset + len * 8
    }

    spec skip_u64_vector {
        pragma verify = false;
    }

    #[test]
    fun test_skip_u64_vector() {
        let vec = vector::empty<u64>();
        vector::push_back(&mut vec, 11111);
        vector::push_back(&mut vec, 22222);
        let u_64 = bcs::to_bytes(&10);
        let vec = bcs::to_bytes(&vec);
        vector::append(&mut vec, u_64);
        // vec : [2, 103, 43, 0, 0, 0, 0, 0, 0, 206, 86, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0]
        assert!(skip_u64_vector(&vec, 0) == 17, 2004);
    }

    // skip vector<u128>
    public fun skip_u128_vector(input: &vector<u8>, offset: u64): u64 {
        let (len, new_offset) = deserialize_len(input, offset);
        can_skip(input, new_offset, len * 16);
        new_offset + len * 16
    }

    spec skip_u128_vector {
        pragma verify = false;
    }

    #[test]
    fun test_skip_u128_vector() {
        let vec = vector::empty<u128>();
        vector::push_back(&mut vec, 11111);
        vector::push_back(&mut vec, 22222);
        let u_64 = bcs::to_bytes(&10);
        let vec = bcs::to_bytes(&vec);
        vector::append(&mut vec, u_64);
        // vec : [2, 103, 43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 206, 86, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0]
        assert!(skip_u128_vector(&vec, 0) == 33, 2003);
    }

    // skip u256
    public fun skip_u256(input: &vector<u8>, offset: u64): u64 {
        can_skip(input, offset, 32);
        offset + 32
    }

    spec skip_u256 {
        pragma verify = false;
    }

    // skip u128
    public fun skip_u128(input: &vector<u8>, offset: u64): u64 {
        can_skip(input, offset, 16);
        offset + 16
    }

    spec skip_u128 {
        pragma verify = false;
    }

    #[test]
    fun test_skip_u128() {
        let u_128: u128 = 100;
        let u_128 = bcs::to_bytes(&u_128);
        let vec = bcs::to_bytes(&x"01020304");
        vector::append(&mut u_128, vec);
        // u_128 : [100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 2, 3, 4]
        assert!(skip_u128(&u_128, 0) == 16, 2005);
    }

    // skip u64
    public fun skip_u64(input: &vector<u8>, offset: u64): u64 {
        can_skip(input, offset, 8);
        offset + 8
    }

    spec skip_u64 {
        pragma verify = false;
    }

    #[test]
    fun test_skip_u64() {
        let u_64: u64 = 100;
        let u_64 = bcs::to_bytes(&u_64);
        let vec = bcs::to_bytes(&x"01020304");
        vector::append(&mut u_64, vec);
        // u_64 : [100, 0, 0, 0, 0, 0, 0, 0, 4, 1, 2, 3, 4]
        assert!(skip_u64(&u_64, 0) == 8, 2006);
    }

    // skip u32
    public fun skip_u32(input: &vector<u8>, offset: u64): u64 {
        can_skip(input, offset, 4);
        offset + 4
    }

    spec skip_u32 {
        pragma verify = false;
    }

    // skip u16
    public fun skip_u16(input: &vector<u8>, offset: u64): u64 {
        can_skip(input, offset, 2);
        offset + 2
    }

    spec skip_u16 {
        pragma verify = false;
    }

    // skip address
    public fun skip_address(input: &vector<u8>, offset: u64): u64 {
        skip_n_bytes(input, offset, 16)
    }

    spec skip_address {
        pragma verify = false;
    }

    #[test]
    fun test_address() {
        let addr: address = @0x1;
        let addr = bcs::to_bytes(&addr);
        let vec = bcs::to_bytes(&x"01020304");
        vector::append(&mut addr, vec);
        // addr :  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 4, 1, 2, 3, 4]
        assert!(skip_address(&addr, 0) == 16, 2006);
    }

    // skip bool
    public fun skip_bool(input: &vector<u8>, offset: u64): u64 {
        can_skip(input, offset, 1);
        offset + 1
    }

    spec skip_bool {
        pragma verify = false;
    }

    fun can_skip(input: &vector<u8>, offset: u64, n: u64) {
        assert!(
            ((offset + n) <= vector::length(input)) && (offset < offset + n),
            error::invalid_state(ERR_INPUT_NOT_LARGE_ENOUGH)
        );
    }

    spec can_skip {
        pragma verify = false;
    }
}