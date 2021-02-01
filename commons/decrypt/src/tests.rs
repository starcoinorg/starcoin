use crate::{decrypt, encrypt};

#[test]
fn test_encryption() {
    let secret = "hello";
    let plain = "world";
    let encrypted = encrypt(secret.as_bytes(), plain.as_bytes());
    assert_ne!(encrypted.as_slice(), plain.as_bytes());

    let decrypted = decrypt(secret.as_bytes(), encrypted.as_slice()).unwrap();
    assert_eq!(decrypted.as_slice(), plain.as_bytes());
}
