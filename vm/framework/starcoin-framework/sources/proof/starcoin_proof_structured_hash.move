module starcoin_framework::starcoin_proof_structured_hash {
    use std::bcs;
    use std::hash;
    use std::vector;

    const STARCOIN_HASH_PREFIX: vector<u8> = b"STARCOIN::";

    public fun hash<MoveValue: store>(structure: vector<u8>, data: &MoveValue): vector<u8> {
        let prefix_hash = hash::sha3_256(concat(&STARCOIN_HASH_PREFIX, structure));
        let bcs_bytes = bcs::to_bytes(data);
        hash::sha3_256(concat(&prefix_hash, bcs_bytes))
    }

    fun concat(v1: &vector<u8>, v2: vector<u8>): vector<u8> {
        let data = *v1;
        vector::append(&mut data, v2);
        data
    }
}