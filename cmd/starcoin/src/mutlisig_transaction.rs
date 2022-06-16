use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use starcoin_crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::multi_ed25519::multi_shard::MultiEd25519SignatureShard;
use starcoin_crypto::multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature};

use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::transaction::authenticator::TransactionAuthenticator;
use starcoin_vm_types::transaction::{RawUserTransaction, SignedUserTransaction};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MultisigTransaction {
    raw_txn: RawUserTransaction,
    /// user who need to sign the txn.
    signers: Vec<Ed25519PublicKey>,
    /// num of signatures needed to fulfill the txn.
    threshold: u8,
    /// collected signatures.
    signatures: HashMap<Ed25519PublicKey, Ed25519Signature>,
}

impl MultisigTransaction {
    pub fn new(raw_txn: RawUserTransaction, signers: Vec<Ed25519PublicKey>, threshold: u8) -> Self {
        Self {
            raw_txn,
            signers,
            threshold,
            signatures: HashMap::new(),
        }
    }

    pub fn raw_txn(&self) -> &RawUserTransaction {
        &self.raw_txn
    }

    pub fn multi_public_key(&self) -> MultiEd25519PublicKey {
        MultiEd25519PublicKey::new(self.signers.clone(), self.threshold).expect("multi public key")
    }

    pub fn collected_signatures(&self) -> &HashMap<Ed25519PublicKey, Ed25519Signature> {
        &self.signatures
    }

    pub fn can_signed_by(&self, key: &Ed25519PublicKey) -> bool {
        self.signer_position(key).is_some()
    }

    fn signer_position(&self, signer: &Ed25519PublicKey) -> Option<u8> {
        let mut found = None;
        for (i, s) in self.signers.iter().enumerate() {
            if s.to_bytes() == signer.to_bytes() {
                found = Some(i as u8);
                break;
            }
        }
        found
    }

    pub fn collect_signature(
        &mut self,
        signer: Ed25519PublicKey,
        signature: Ed25519Signature,
    ) -> bool {
        if !self.can_signed_by(&signer) {
            return false;
        }
        self.signatures.insert(signer, signature);
        true
    }

    pub fn into_signed_txn(self) -> Result<SignedUserTransaction> {
        let mut sigs = vec![];
        for (key, signature) in self.signatures.iter() {
            let pos = self
                .signer_position(key)
                .expect("should included in signers");
            sigs.push((signature.clone(), pos));
        }
        let multi_sig = MultiEd25519Signature::new(sigs)?;
        let multi_key = self.multi_public_key();
        Ok(SignedUserTransaction::multi_ed25519(
            self.raw_txn,
            multi_key,
            multi_sig,
        ))
    }
}

pub struct RawTxnMultiSign {
    pub txn: RawUserTransaction,
    pub signatures: Option<MultiEd25519SignatureShard>,
}

pub fn read_multisig_existing_signatures(file_input: &Path) -> Result<RawTxnMultiSign> {
    let txn: SignedUserTransaction = bcs_ext::from_bytes(&std::fs::read(file_input)?)?;

    let existing_signatures = match txn.authenticator() {
        TransactionAuthenticator::Ed25519 { .. } => {
            bail!("expect a multisig txn in file {}", file_input.display());
        }
        TransactionAuthenticator::MultiEd25519 {
            public_key,
            signature,
        } => MultiEd25519SignatureShard::new(signature, *public_key.threshold()),
    };

    Ok(RawTxnMultiSign {
        txn: txn.raw_txn().clone(),
        signatures: Some(existing_signatures),
    })
}

pub fn sign_multisig_txn_to_file(
    sender: AccountAddress,
    multisig_public_key: MultiEd25519PublicKey,
    existing_signatures: Option<MultiEd25519SignatureShard>,
    partial_signed_txn: SignedUserTransaction,
    output_dir: PathBuf,
) -> Result<PathBuf> {
    let my_signatures = if let TransactionAuthenticator::MultiEd25519 { signature, .. } =
        partial_signed_txn.authenticator()
    {
        MultiEd25519SignatureShard::new(signature, *multisig_public_key.threshold())
    } else {
        unreachable!()
    };

    // merge my signatures with existing signatures of other participants.
    let merged_signatures = {
        let mut signatures = vec![];
        if let Some(s) = existing_signatures {
            signatures.push(s);
        }
        signatures.push(my_signatures);
        MultiEd25519SignatureShard::merge(signatures)?
    };
    eprintln!(
        "mutlisig txn(address: {}, threshold: {}): {} signatures collected",
        sender,
        merged_signatures.threshold(),
        merged_signatures.signatures().len()
    );
    if !merged_signatures.is_enough() {
        eprintln!(
            "still require {} signatures",
            merged_signatures.threshold() as usize - merged_signatures.signatures().len()
        );
    } else {
        eprintln!("enough signatures collected for the multisig txn, txn can be submitted now");
    }

    // construct the signed txn with merged signatures.
    let signed_txn = {
        let authenticator = TransactionAuthenticator::MultiEd25519 {
            public_key: multisig_public_key,
            signature: merged_signatures.into(),
        };
        SignedUserTransaction::new(partial_signed_txn.into_raw_transaction(), authenticator)
    };

    // output the txn, send this to other participants to sign, or just submit it.
    let output_file = {
        let mut output_dir = output_dir;
        // use hash's as output file name
        let file_name = signed_txn.crypto_hash().to_hex();
        output_dir.push(file_name);
        output_dir.set_extension("multisig-txn");
        output_dir
    };
    let mut file = File::create(output_file.clone())?;
    // write txn to file
    bcs_ext::serialize_into(&mut file, &signed_txn)?;
    Ok(output_file)
}
