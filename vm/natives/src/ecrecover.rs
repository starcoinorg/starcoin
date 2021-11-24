// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use arrayref::array_ref;
use libsecp256k1::curve::Scalar;
use libsecp256k1::{recover, Message, PublicKey, RecoveryId, Signature};
use log::debug;
use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeResult},
    pop_arg,
    values::Value,
};
use smallvec::smallvec;
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;
use tiny_keccak::Hasher;

const HASH_LENGTH: usize = 32;
const SCALAR_LENGTH: usize = 32;
const SIG_LENGTH: usize = SCALAR_LENGTH + SCALAR_LENGTH;
const SIG_REC_LENGTH: usize = SIG_LENGTH + 1;
const ZERO_ADDR: [u8; 0] = [0; 0];

/// recover address from signature, if recover fail, return an zero address
pub fn native_ecrecover(
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 2);

    // second arg is sig
    let sig_arg = pop_arg!(arguments, Vec<u8>);
    // first arg is hash
    let hash_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::ECRECOVER as u8,
        hash_arg.len(),
    );
    if hash_arg.len() != HASH_LENGTH || sig_arg.len() != SIG_REC_LENGTH {
        debug!("ecrecover failed, invalid hash or sig");
        return Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(ZERO_ADDR)],
        ));
    }

    if sig_arg.len() != SIG_REC_LENGTH {
        return Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(ZERO_ADDR)],
        ));
    }

    match ecrecover_address(
        array_ref![hash_arg, 0, HASH_LENGTH],
        array_ref![sig_arg, 0, SIG_REC_LENGTH],
    ) {
        Ok(output) => Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(output)])),
        Err(err) => {
            debug!("ecrecover error: {:?}", err);
            Ok(NativeResult::ok(
                cost,
                smallvec![Value::vector_u8(ZERO_ADDR)],
            ))
        }
    }
}

pub(crate) fn keccak(input: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut keccak = tiny_keccak::Keccak::v256();
    keccak.update(input);
    keccak.finalize(&mut output);
    output
}

pub(crate) fn pubkey_to_address(key: &PublicKey) -> Vec<u8> {
    let ret = key.serialize();
    let ret = keccak(&ret[1..]);
    ret[12..].to_vec()
}

fn ecrecover_from_vrs(
    h: &[u8; HASH_LENGTH],
    v: u8,
    r: &[u8; SCALAR_LENGTH],
    s: &[u8; SCALAR_LENGTH],
) -> Result<PublicKey, libsecp256k1::Error> {
    if v != 0 && v != 1 {
        return Err(libsecp256k1::Error::InvalidSignature);
    }
    let message = Message::parse(h);
    let rec_id = RecoveryId::parse(v)?;

    let sig = {
        let mut r_scalar = Scalar::default();
        let mut s_scalar = Scalar::default();

        // It's okay for the signature to overflow here, it's checked below.
        let overflowed_r = r_scalar.set_b32(r);
        let overflowed_s = s_scalar.set_b32(s);

        if bool::from(overflowed_r | overflowed_s) {
            return Err(libsecp256k1::Error::InvalidSignature);
        }
        Signature {
            r: r_scalar,
            s: s_scalar,
        }
    };
    recover(&message, &sig, &rec_id)
}

///recover PublicKey from elliptic curve signature
fn ecrecover(h: &[u8; 32], sig: &[u8; SIG_REC_LENGTH]) -> Result<PublicKey, libsecp256k1::Error> {
    let v = sig[64];
    // Transform yellow paper V from 27/28 to 0/1
    let v = if v == 27 || v == 28 {
        v.saturating_sub(27)
    } else {
        v
    };
    ecrecover_from_vrs(
        h,
        v,
        array_ref![sig, 0, SCALAR_LENGTH],
        array_ref![sig, 32, SCALAR_LENGTH],
    )
}

fn ecrecover_address(
    h: &[u8; 32],
    sig: &[u8; SIG_REC_LENGTH],
) -> Result<Vec<u8>, libsecp256k1::Error> {
    Ok(pubkey_to_address(&ecrecover(h, sig)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;
    use libsecp256k1::{PublicKey, SecretKey};
    use rand::rngs::OsRng;
    use starcoin_crypto::HashValue;

    #[test]
    fn empty_input() {
        let hash = [0u8; HASH_LENGTH];
        let sig = [0u8; SIG_REC_LENGTH];

        let output = ecrecover(&hash, &sig);
        assert!(output.is_err());
    }

    #[test]
    fn test_random() {
        let mut rng = OsRng;
        let data = HashValue::random();
        let message = Message::parse(&data);
        let seckey = SecretKey::random(&mut rng);
        let pubkey = PublicKey::from_secret_key(&seckey);
        let address = pubkey_to_address(&pubkey);
        let (sig, rec_id) = libsecp256k1::sign(&message, &seckey);
        let mut sign_with_rec_id = sig.serialize().to_vec();
        sign_with_rec_id.push(rec_id.serialize() + 27);

        let output_vrs = ecrecover_from_vrs(&data, rec_id.serialize(), &sig.r.b32(), &sig.s.b32());
        let output = ecrecover(&data, array_ref![sign_with_rec_id, 0, SIG_REC_LENGTH]);
        assert_eq!(output, output_vrs);
        assert!(output.is_ok());

        let recover_pubkey = output.unwrap();
        let recover_address = pubkey_to_address(&recover_pubkey);
        assert_eq!(address, recover_address);
    }

    // test case from https://github.com/MetaMask/eth-sig-util/blob/fb880d8d95abf3f3bc6d1b0bcf8b8933e9718c5a/src/personal-sign.test.ts
    #[test]
    fn test_helloworld() {
        let private_key: Vec<u8> =
            FromHex::from_hex("4af1bceebf7f3634ec3cff8a2c38e51178d5d4ce585c52d6043e5e2cc3418bb0")
                .unwrap();

        let seckey = SecretKey::parse(array_ref![private_key, 0, 32]).unwrap();
        let pubkey = PublicKey::from_secret_key(&seckey);
        let address_from_hex: Vec<u8> =
            FromHex::from_hex("29c76e6ad8f28bb1004902578fb108c507be341b").unwrap();
        let address = pubkey_to_address(&pubkey);
        assert_eq!(address_from_hex, address, "address miss match");
        let org_msg = "Hello, world!";
        let prefix = format!("\u{0019}Ethereum Signed Message:\n{}", org_msg.len());
        let mut data = vec![];

        data.extend_from_slice(prefix.as_bytes());
        data.extend_from_slice(org_msg.as_bytes());

        let msg_hash_from_hex: Vec<u8> =
            FromHex::from_hex("b453bd4e271eed985cbab8231da609c4ce0a9cf1f763b6c1594e76315510e0f1")
                .unwrap();
        let msg_hash = keccak(data.as_slice());

        assert_eq!(msg_hash_from_hex, msg_hash, "hash miss match");

        let message = Message::parse(&msg_hash);
        let hello_sig_from_hex:Vec<u8> = FromHex::from_hex("90a938f7457df6e8f741264c32697fc52f9a8f867c52dd70713d9d2d472f2e415d9c94148991bbe1f4a1818d1dff09165782749c877f5cf1eff4ef126e55714d1c").unwrap();

        let (hello_sig, rec_id) = libsecp256k1::sign(&message, &seckey);
        let mut sign_with_rec_id = hello_sig.serialize().to_vec();
        sign_with_rec_id.push(rec_id.serialize() + 27);
        assert_eq!(sign_with_rec_id.len(), SIG_REC_LENGTH);
        assert_eq!(hello_sig_from_hex, sign_with_rec_id, "signature miss match");

        let recover_pubkey =
            ecrecover(&msg_hash, array_ref![hello_sig_from_hex, 0, SIG_REC_LENGTH]).unwrap();

        assert_eq!(pubkey, recover_pubkey, "pubkey miss match");
        let recover_address = pubkey_to_address(&recover_pubkey);
        assert_eq!(address, recover_address, "address miss match");
    }
}
