// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use names::{Generator, Name};

pub(crate) fn save_config<T, P>(c: &T, output_file: P) -> Result<()>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<Path>,
{
    let mut file = File::create(output_file)?;
    file.write_all(to_toml(c)?.as_bytes())?;
    Ok(())
}

pub(crate) fn to_toml<T>(c: &T) -> Result<String>
where
    T: Serialize + DeserializeOwned,
{
    // fix toml table problem, see https://github.com/alexcrichton/toml-rs/issues/142
    let c = toml::value::Value::try_from(c)?;
    Ok(toml::to_string(&c)?)
}

pub(crate) fn load_config<T, P>(path: P) -> Result<T>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<Path>,
{
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    parse(&contents)
}

fn parse<T>(serialized: &str) -> Result<T>
where
    T: Serialize + DeserializeOwned,
{
    Ok(toml::from_str(serialized)?)
}

pub(crate) fn save_key<P>(key: &[u8], output_file: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let contents: String = hex::encode(key);
    let mut file = open_key_file(output_file)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub(crate) fn decode_key(hex_str: &str) -> Result<(Ed25519PrivateKey, Ed25519PublicKey)> {
    let bytes_out: Vec<u8> = hex::decode(hex_str)?;
    let pri_key = Ed25519PrivateKey::try_from(bytes_out.as_slice())?;
    let pub_key = Ed25519PublicKey::from(&pri_key);
    Ok((pri_key, pub_key))
}

pub(crate) fn load_key<P: AsRef<Path>>(path: P) -> Result<(Ed25519PrivateKey, Ed25519PublicKey)> {
    let content = fs::read_to_string(path)?;
    decode_key(content.as_str())
}

pub(crate) fn gen_keypair() -> (Ed25519PrivateKey, Ed25519PublicKey) {
    let mut gen = KeyGen::from_os_rng();
    gen.generate_keypair()
}

/// Opens a file containing a secret key in write mode.
#[cfg(unix)]
fn open_key_file<P>(path: P) -> io::Result<fs::File>
where
    P: AsRef<Path>,
{
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

/// Opens a file containing a secret key in write mode.
#[cfg(not(unix))]
fn open_key_file<P>(path: P) -> Result<fs::File, io::Error>
where
    P: AsRef<Path>,
{
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
}

const NODE_NAME_MAX_LENGTH: usize = 64;
/// Generate a valid random name for the node
pub(crate) fn generate_node_name() -> String {
    loop {
        let node_name = Generator::with_naming(Name::Numbered)
            .next()
            .expect("RNG is available on all supported platforms; qed");
        let count = node_name.chars().count();

        if count < NODE_NAME_MAX_LENGTH {
            return node_name;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::decode_key;
    use network_p2p_types::peer_id::PeerId;

    fn parse_private_keys(raw_keys: &str) -> Vec<&str> {
        raw_keys
            .lines()
            .flat_map(|line| line.split(|c| matches!(c, ',' | ';')))
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .collect()
    }

    #[test]
    #[ignore = "manual utility test; set STC_PEER_PRIVATE_KEYS to a comma/semicolon/newline separated list of hex private keys"]
    fn print_peer_id_addresses_from_private_keys() {
        let raw_keys = std::env::var("STC_PEER_PRIVATE_KEYS").expect(
            "missing STC_PEER_PRIVATE_KEYS; provide comma/semicolon/newline separated hex private keys",
        );
        
        let keys: Vec<&str> = parse_private_keys(&raw_keys);
        for (index, private_key_hex) in keys.iter().enumerate() {
            let (_private_key, public_key) =
                decode_key(private_key_hex).expect("peer private key should decode successfully");
            let peer_id = PeerId::from_ed25519_public_key(public_key.clone());

            println!(
                "peer[{index}] public_key={} peer_id={} p2p_suffix=/p2p/{}",
                hex::encode(public_key.to_bytes()),
                peer_id,
                peer_id
            );
        }

        // Print the kubectl command to create/update the node-keys secret for main network
        let joined = keys.join("; ");
        println!();
        println!("--- kubectl create secret command (main network) ---");
        println!(
            "kubectl -n starcoin-main create secret generic node-keys \\\n  --from-literal=node-keys='{}' \\\n  --dry-run=client -o yaml | kubectl apply -f -",
            joined
        );
    }
}
