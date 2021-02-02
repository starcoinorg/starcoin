use super::*;
use crate::multi_ed25519::multi_shard::MultiEd25519SignatureShard;
use crate::test_utils::{TestDiemCrypto, TEST_SEED};
use crate::{Signature, ValidCryptoMaterial, ValidCryptoMaterialStringExt};
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::convert::TryFrom;

fn generate_shards(n: usize, threshold: u8) -> Vec<MultiEd25519KeyShard> {
    let mut rng = StdRng::from_seed(TEST_SEED);
    MultiEd25519KeyShard::generate(&mut rng, n, threshold).unwrap()
}

static MESSAGE: Lazy<TestDiemCrypto> = Lazy::new(|| TestDiemCrypto("Test Message".to_string()));
fn message() -> &'static TestDiemCrypto {
    &MESSAGE
}

#[test]
pub fn test_to_string_by_read_seed() {
    let mut seed_rng = rand::rngs::OsRng;
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng = StdRng::from_seed(seed_buf);
    let shards = MultiEd25519KeyShard::generate(&mut rng, 3, 2).unwrap();
    for shard in shards {
        let hex_str = shard.to_encoded_string().unwrap();
        let shard2 = MultiEd25519KeyShard::from_encoded_string(hex_str.as_str()).unwrap();
        assert_eq!(shard, shard2);
        assert!(shard.to_encoded_string().is_ok());
        // println!(
        //     "index: {}\npublic_key:\n{} \nimport_key:\n{}\n",
        //     shard.index,
        //     shard.public_key().to_encoded_string().unwrap(),
        //     shard.to_encoded_string().unwrap(),
        // )
    }
}

#[test]
pub fn test_multi_private_key_shard_serialize() {
    let shards = generate_shards(3, 2);
    for shard in shards {
        let bytes = shard.to_bytes();
        let shard2 = MultiEd25519KeyShard::try_from(bytes.as_slice()).unwrap();
        assert_eq!(shard, shard2)
    }
}

#[test]
pub fn test_shard_sign_and_verify() {
    let shards = generate_shards(3, 2);
    let msg = message();

    let signatures = shards
        .iter()
        .map(|shard| shard.sign(msg))
        .collect::<Vec<_>>();
    let public_key = shards[0].public_key();
    for signature in signatures.as_slice() {
        assert!(
            signature.verify(msg, &public_key).is_ok(),
            "verify msg by signature {:?} fail.",
            signature
        );
        assert!(!signature.is_enough());
    }
    let signature2of3 = MultiEd25519SignatureShard::merge(signatures[..2].to_vec()).unwrap();
    assert!(signature2of3.is_enough());
    let multi_signature: MultiEd25519Signature = signature2of3.into();
    multi_signature.verify(msg, &public_key).unwrap();
}
