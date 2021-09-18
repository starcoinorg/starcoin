use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead};
use anyhow::{bail, format_err, Result};
use byteorder::{ReadBytesExt, WriteBytesExt};
use rand::RngCore;
use std::io::{Cursor, Read, Write};

pub const PBKDF2_DEFAULT_ITERATIONS: usize = 1000;
pub const PBKDF2_SALT_SIZE: usize = 32;
pub const AES_NONCE_SIZE: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct KeyDerivationParams {
    pbkdf2_iterations: u32,
    pbkdf2_salt: [u8; PBKDF2_SALT_SIZE],
}

impl KeyDerivationParams {
    pub fn generate() -> Self {
        let mut salt = [0u8; PBKDF2_SALT_SIZE];
        rand::thread_rng().fill_bytes(&mut salt);
        Self {
            pbkdf2_iterations: PBKDF2_DEFAULT_ITERATIONS as u32,
            pbkdf2_salt: salt,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct EncryptionParams {
    nonce: [u8; AES_NONCE_SIZE],
}

impl EncryptionParams {
    /// Generate encryption params
    pub fn generate() -> Self {
        let mut nonce = [0u8; AES_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce);
        Self { nonce }
    }
}

const META_LEN: usize = 4usize + PBKDF2_SALT_SIZE + AES_NONCE_SIZE;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Meta {
    key_derive_params: KeyDerivationParams,

    encryption_params: EncryptionParams,
}
impl Meta {
    pub fn generate() -> Self {
        Self {
            key_derive_params: KeyDerivationParams::generate(),
            encryption_params: EncryptionParams::generate(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::with_capacity(META_LEN));
        buf.write_u32::<byteorder::BigEndian>(self.key_derive_params.pbkdf2_iterations)
            .expect("should never fail");
        buf.write_all(&self.key_derive_params.pbkdf2_salt)
            .expect("should never fail");
        buf.write_all(&self.encryption_params.nonce)
            .expect("should never fail");
        buf.into_inner()
    }

    pub fn decode(buf: &[u8]) -> Result<Self> {
        let mut buf = Cursor::new(buf);
        let iterations = buf.read_u32::<byteorder::BigEndian>()?;
        let mut salt = [0u8; PBKDF2_SALT_SIZE];
        buf.read_exact(&mut salt)?;
        let mut nonce = [0u8; AES_NONCE_SIZE];
        buf.read_exact(&mut nonce)?;
        Ok(Self {
            key_derive_params: KeyDerivationParams {
                pbkdf2_salt: salt,
                pbkdf2_iterations: iterations,
            },
            encryption_params: EncryptionParams { nonce },
        })
    }
}

fn derive_key(derivation_param: &KeyDerivationParams, secret: &[u8]) -> [u8; 32] {
    // 256-bit derived key
    let mut dk = [0u8; 32];
    // use secret to derive a key to encrypt plaintext
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        secret,
        &derivation_param.pbkdf2_salt,
        derivation_param.pbkdf2_iterations as usize,
        &mut dk,
    );
    dk
}

fn aes_encrypt(encryption_param: &EncryptionParams, key: [u8; 32], plain: &[u8]) -> Vec<u8> {
    let key = GenericArray::from(key);
    let nonce = GenericArray::clone_from_slice(&encryption_param.nonce);
    let cipher = aes_gcm::Aes256Gcm::new(&key);
    cipher
        .encrypt(&nonce, plain)
        .expect("encryption should never failure!")
}
fn aes_decrypt(
    encryption_param: &EncryptionParams,
    key: [u8; 32],
    encrypted: &[u8],
) -> Result<Vec<u8>> {
    let key = GenericArray::from(key);
    let nonce = GenericArray::clone_from_slice(&encryption_param.nonce);
    let cipher = aes_gcm::Aes256Gcm::new(&key);
    match cipher.decrypt(&nonce, encrypted) {
        Ok(s) => Ok(s),
        Err(e) => Err(format_err!("decrypt error:{:?}", e)),
    }
}

pub fn encrypt(secret: &[u8], plain: &[u8]) -> Vec<u8> {
    let meta = Meta::generate();
    // 256-bit derived key
    let dk = derive_key(&meta.key_derive_params, secret);
    let mut ciphertext = aes_encrypt(&meta.encryption_params, dk, plain);
    let mut result = meta.encode();
    result.append(&mut ciphertext);
    result
}

pub fn decrypt(secret: &[u8], encrypted: &[u8]) -> Result<Vec<u8>> {
    if encrypted.len() <= META_LEN {
        bail!("invalid encrypted data");
    }
    let meta = Meta::decode(&encrypted[0..META_LEN])?;
    let crypted = &encrypted[META_LEN..];

    let dk = derive_key(&meta.key_derive_params, secret);
    aes_decrypt(&meta.encryption_params, dk, crypted)
}

#[cfg(test)]
mod tests;
