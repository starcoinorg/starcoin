use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use starcoin_crypto::multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature};
use starcoin_vm_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::collections::HashMap;
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
