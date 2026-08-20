// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    block::{Block, BlockBody, BlockHeader, BlockNumber},
    genesis_config::ChainId,
    identifier::Identifier,
    language_storage::{ModuleId, CORE_CODE_ADDRESS},
    transaction::{
        authenticator::{AuthenticationKey, TransactionAuthenticator},
        RawUserTransaction, SignedUserTransaction, TransactionInfo, TransactionPayload,
    },
    vm_error::KeptVMStatus,
    U256,
};
use anyhow::{ensure, Result};
use serde::Serialize;
use starcoin_crypto::{ed25519::Ed25519PublicKey, HashValue};
use starcoin_vm_types::{
    account_config::{stc_type_tag, STC_TOKEN_CODE_STR},
    transaction::ScriptFunction,
};

/// Release inputs. Each height/key pair must be fixed together before building the immutable
/// artifact promoted through Halley, Barnard, and Main.
pub const HALLEY_BLOCK_PERMIT_HEIGHT: BlockNumber = 2_894_400;
pub const HALLEY_BLOCK_PERMIT_AUTH_KEY: AuthenticationKey = AuthenticationKey::new([
    0x15, 0x7b, 0x42, 0xa6, 0x65, 0x60, 0xe8, 0x31, 0x44, 0xde, 0xec, 0xdb, 0x43, 0xd5, 0xba, 0xc7,
    0x0a, 0x5e, 0x11, 0x6b, 0x09, 0x38, 0xd2, 0x77, 0x19, 0x2d, 0xe7, 0xd0, 0x16, 0x17, 0xe2, 0xac,
]);
pub const BARNARD_BLOCK_PERMIT_HEIGHT: BlockNumber = 19_667_300;
pub const BARNARD_BLOCK_PERMIT_AUTH_KEY: AuthenticationKey = AuthenticationKey::new([
    0xca, 0x72, 0xd0, 0xee, 0x15, 0xcf, 0x78, 0xc1, 0x23, 0x22, 0x18, 0xc8, 0xd4, 0xc2, 0x41, 0xc8,
    0x7e, 0x6c, 0xdc, 0x69, 0x53, 0xfe, 0xa8, 0x13, 0xc7, 0xbe, 0x21, 0x91, 0xf9, 0x38, 0x8e, 0x67,
]);
pub const MAINNET_BLOCK_PERMIT_HEIGHT: BlockNumber = 32_165_300;
pub const MAINNET_BLOCK_PERMIT_AUTH_KEY: AuthenticationKey = AuthenticationKey::new([
    0x6e, 0xa5, 0x44, 0x07, 0x8e, 0xfc, 0xba, 0xc8, 0x1e, 0x9d, 0x61, 0x97, 0x09, 0x0b, 0x1a, 0x10,
    0x6c, 0xd6, 0x9f, 0x2f, 0xef, 0x83, 0x20, 0x4a, 0x2d, 0xde, 0xcf, 0x05, 0x98, 0x06, 0x01, 0xcd,
]);

const BLOCK_PERMIT_DOMAIN: &[u8] = b"STARCOIN_BLOCK_PERMIT_V1";

/// Non-serialized consensus policy. There is no production CLI/TOML override for these values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockPermitPolicy {
    trusted_chain_id: Option<ChainId>,
    activation_height: BlockNumber,
    authentication_key: AuthenticationKey,
}

impl BlockPermitPolicy {
    /// Resolve a release policy only after the caller has matched a trusted built-in network
    /// identity. Node startup verifies that identity against stored genesis before constructing
    /// services; custom networks must use `disabled` even if they claim a built-in chain id.
    pub fn for_trusted_builtin(chain_id: ChainId) -> Self {
        assert!(
            chain_id.is_halley() || chain_id.is_barnard() || chain_id.is_main(),
            "block permit release policy requires halley, barnard, or main"
        );
        let policy = match chain_id.id() {
            253 => Self {
                trusted_chain_id: Some(chain_id),
                activation_height: HALLEY_BLOCK_PERMIT_HEIGHT,
                authentication_key: HALLEY_BLOCK_PERMIT_AUTH_KEY,
            },
            251 => Self {
                trusted_chain_id: Some(chain_id),
                activation_height: BARNARD_BLOCK_PERMIT_HEIGHT,
                authentication_key: BARNARD_BLOCK_PERMIT_AUTH_KEY,
            },
            1 => Self {
                trusted_chain_id: Some(chain_id),
                activation_height: MAINNET_BLOCK_PERMIT_HEIGHT,
                authentication_key: MAINNET_BLOCK_PERMIT_AUTH_KEY,
            },
            _ => unreachable!("supported chain id checked above"),
        };
        assert!(
            policy.release_inputs_consistent(),
            "block permit height and authentication key must be configured together"
        );
        policy
    }

    pub const fn disabled() -> Self {
        Self {
            trusted_chain_id: None,
            activation_height: BlockNumber::MAX,
            authentication_key: AuthenticationKey::new(AuthenticationKey::DUMMY_KEY),
        }
    }

    /// Explicit constructor for focused cross-crate tests.
    #[cfg(any(test, feature = "block-permit-test-utils"))]
    pub fn new_for_test(
        activation_height: BlockNumber,
        authentication_key: AuthenticationKey,
    ) -> Self {
        Self::new_for_chain_test(ChainId::new(1), activation_height, authentication_key)
    }

    /// Explicit constructor for focused non-main built-in network tests.
    #[cfg(any(test, feature = "block-permit-test-utils"))]
    pub fn new_for_chain_test(
        trusted_chain_id: ChainId,
        activation_height: BlockNumber,
        authentication_key: AuthenticationKey,
    ) -> Self {
        Self {
            trusted_chain_id: Some(trusted_chain_id),
            activation_height,
            authentication_key,
        }
    }

    pub const fn trusted_chain_id(self) -> Option<ChainId> {
        self.trusted_chain_id
    }

    pub const fn activation_height(self) -> BlockNumber {
        self.activation_height
    }

    pub fn release_configured(self) -> bool {
        self.trusted_chain_id.is_some()
            && self.activation_height != BlockNumber::MAX
            && !self.authentication_key.is_dummy()
    }

    fn release_inputs_consistent(self) -> bool {
        self.trusted_chain_id.is_none()
            || ((self.activation_height != BlockNumber::MAX) != self.authentication_key.is_dummy())
    }

    pub fn is_active(self, block_number: BlockNumber) -> bool {
        self.release_configured() && block_number >= self.activation_height
    }

    fn authorization_generation(self, block_number: BlockNumber) -> u64 {
        u64::from(self.is_active(block_number))
    }

    pub fn authentication_key(self, block_number: BlockNumber) -> Option<AuthenticationKey> {
        self.is_active(block_number)
            .then_some(self.authentication_key)
    }
}

/// Local fork-choice value. Construct this only from a head whose blocks have passed local
/// validation; peer advertisements must use `AdvertisedSyncRank` instead.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValidatedChainQuality(u64, U256);

/// Peer scheduling value. This may be based on an untrusted claimed height and must never be used
/// directly to select or retain the local main chain.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AdvertisedSyncRank(u64, U256);

pub fn validated_chain_quality(
    policy: BlockPermitPolicy,
    locally_validated_head_number: BlockNumber,
    total_difficulty: U256,
) -> ValidatedChainQuality {
    ValidatedChainQuality(
        policy.authorization_generation(locally_validated_head_number),
        total_difficulty,
    )
}

pub fn advertised_sync_rank(
    policy: BlockPermitPolicy,
    peer_claimed_head_number: BlockNumber,
    advertised_total_difficulty: U256,
) -> AdvertisedSyncRank {
    AdvertisedSyncRank(
        policy.authorization_generation(peer_claimed_head_number),
        advertised_total_difficulty,
    )
}

#[derive(Serialize)]
struct BlockPermitMessageV1 {
    chain_id: u8,
    number: u64,
    parent_hash: HashValue,
    timestamp_millis: u64,
    author: AccountAddress,
    difficulty: U256,
    block_accumulator_root: HashValue,
    state_root: HashValue,
    gas_used: u64,
    body_without_permit_hash: HashValue,
}

#[allow(clippy::too_many_arguments)]
pub fn block_permit_digest(
    chain_id: ChainId,
    number: BlockNumber,
    parent_hash: HashValue,
    timestamp_millis: u64,
    author: AccountAddress,
    difficulty: U256,
    block_accumulator_root: HashValue,
    state_root: HashValue,
    gas_used: u64,
    body_without_permit: &BlockBody,
) -> Result<HashValue> {
    let encoded = block_permit_message_bytes(
        chain_id,
        number,
        parent_hash,
        timestamp_millis,
        author,
        difficulty,
        block_accumulator_root,
        state_root,
        gas_used,
        body_without_permit,
    )?;
    let mut signing_bytes = Vec::new();
    signing_bytes.extend_from_slice(BLOCK_PERMIT_DOMAIN);
    signing_bytes.push(0);
    signing_bytes.extend_from_slice(&encoded);
    Ok(HashValue::sha3_256_of(&signing_bytes))
}

#[allow(clippy::too_many_arguments)]
fn block_permit_message_bytes(
    chain_id: ChainId,
    number: BlockNumber,
    parent_hash: HashValue,
    timestamp_millis: u64,
    author: AccountAddress,
    difficulty: U256,
    block_accumulator_root: HashValue,
    state_root: HashValue,
    gas_used: u64,
    body_without_permit: &BlockBody,
) -> Result<Vec<u8>> {
    let message = BlockPermitMessageV1 {
        chain_id: chain_id.id(),
        number,
        parent_hash,
        timestamp_millis,
        author,
        difficulty,
        block_accumulator_root,
        state_root,
        gas_used,
        body_without_permit_hash: body_without_permit.hash(),
    };
    bcs_ext::to_bytes(&message)
}

pub fn build_block_permit_raw_transaction(
    public_key: &Ed25519PublicKey,
    author: AccountAddress,
    digest: HashValue,
    chain_id: ChainId,
) -> Result<RawUserTransaction> {
    let payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(CORE_CODE_ADDRESS, Identifier::new("TransferScripts")?),
        Identifier::new("peer_to_peer_with_metadata_v2")?,
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&author)?,
            bcs_ext::to_bytes(&0u128)?,
            bcs_ext::to_bytes(&digest.to_vec())?,
        ],
    ));
    let authentication_key = AuthenticationKey::ed25519(public_key);
    Ok(RawUserTransaction::new(
        authentication_key.derived_address(),
        0,
        payload,
        0,
        0,
        0,
        chain_id,
        STC_TOKEN_CODE_STR.to_string(),
    ))
}

/// Validate the final body transaction as an active block permit. Pre-activation blocks are left
/// byte-for-byte on the existing path.
pub fn validate_block_permit(
    policy: BlockPermitPolicy,
    trusted_chain_id: ChainId,
    parent_header: &BlockHeader,
    block: &Block,
) -> Result<()> {
    if !policy.is_active(block.header().number()) {
        return Ok(());
    }

    let header = block.header();
    ensure!(
        policy.trusted_chain_id() == Some(trusted_chain_id),
        "active block permit policy does not match trusted chain"
    );
    ensure!(
        parent_header.chain_id() == trusted_chain_id,
        "block permit parent chain id mismatch"
    );
    ensure!(
        header.chain_id() == trusted_chain_id,
        "block permit child chain id mismatch"
    );
    ensure!(
        header.parent_hash() == parent_header.id(),
        "block permit parent hash mismatch"
    );
    ensure!(
        header.number() == parent_header.number().saturating_add(1),
        "block permit block number mismatch"
    );
    ensure!(
        header.author_auth_key().is_none(),
        "active block author_auth_key must be None"
    );
    ensure!(
        block.body.hash() == header.body_hash(),
        "active block body hash mismatch"
    );

    let (permit, ordinary_transactions) = block
        .body
        .transactions
        .split_last()
        .ok_or_else(|| anyhow::format_err!("active block is missing its final permit"))?;
    let (public_key, authentication_key) = match permit.authenticator() {
        TransactionAuthenticator::Ed25519 { public_key, .. } => {
            let authentication_key = AuthenticationKey::ed25519(&public_key);
            (public_key, authentication_key)
        }
        TransactionAuthenticator::MultiEd25519 { .. } => {
            anyhow::bail!("active block permit must use Ed25519")
        }
    };

    let expected_authentication_key = policy
        .authentication_key(header.number())
        .expect("active policy must have an authentication key");
    ensure!(
        authentication_key == expected_authentication_key,
        "active block permit authentication key mismatch"
    );
    ensure!(
        permit.sender() == authentication_key.derived_address(),
        "active block permit sender mismatch"
    );

    let body_without_permit =
        BlockBody::new(ordinary_transactions.to_vec(), block.body.uncles.clone());
    let digest = block_permit_digest(
        header.chain_id(),
        header.number(),
        header.parent_hash(),
        header.timestamp(),
        header.author(),
        header.difficulty(),
        header.block_accumulator_root(),
        header.state_root(),
        header.gas_used(),
        &body_without_permit,
    )?;
    let expected_raw =
        build_block_permit_raw_transaction(&public_key, header.author(), digest, trusted_chain_id)?;
    ensure!(
        permit.raw_txn() == &expected_raw,
        "active block permit envelope mismatch"
    );
    permit.clone().check_signature()?;
    Ok(())
}

pub fn active_block_permit(
    policy: BlockPermitPolicy,
    block: &Block,
) -> Option<&SignedUserTransaction> {
    policy
        .is_active(block.header().number())
        .then(|| block.body.transactions.last())
        .flatten()
}

/// The permit is committed to the transaction accumulator without entering the VM.
pub fn block_permit_transaction_info(
    permit: &SignedUserTransaction,
    state_root: HashValue,
) -> TransactionInfo {
    TransactionInfo::new(permit.id(), state_root, &[], 0, KeptVMStatus::Executed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockHeaderExtra;
    use starcoin_crypto::{
        ed25519::{Ed25519PrivateKey, Ed25519Signature},
        multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
        PrivateKey, SigningKey,
    };
    use std::convert::TryFrom;

    const ACTIVATION_HEIGHT: BlockNumber = 10;
    const PRE_ACTIVATION_HEIGHT: BlockNumber = 9;
    const POST_ACTIVATION_HEIGHT: BlockNumber = 11;

    struct Fixture {
        policy: BlockPermitPolicy,
        parent: BlockHeader,
        block: Block,
        private_key: Ed25519PrivateKey,
        digest: HashValue,
        message_bytes: Vec<u8>,
    }

    fn fixture() -> Fixture {
        let chain_id = ChainId::new(1);
        let private_key = Ed25519PrivateKey::try_from(&[7u8; 32][..]).unwrap();
        let public_key = private_key.public_key();
        let authentication_key = AuthenticationKey::ed25519(&public_key);
        let policy = BlockPermitPolicy::new_for_test(ACTIVATION_HEIGHT, authentication_key);
        let author = AccountAddress::new([0x11; 16]);
        let parent = BlockHeader::new(
            HashValue::new([1; 32]),
            1_234_000,
            PRE_ACTIVATION_HEIGHT,
            AccountAddress::new([0x22; 16]),
            HashValue::new([2; 32]),
            HashValue::new([3; 32]),
            HashValue::new([4; 32]),
            12,
            U256::from(1_000u64),
            HashValue::new([5; 32]),
            chain_id,
            9,
            BlockHeaderExtra::new([1, 2, 3, 4]),
        );
        let body_without_permit = BlockBody::new(vec![], None);
        let timestamp = 1_234_567;
        let difficulty = U256::from(42u64);
        let block_accumulator_root = HashValue::new([7; 32]);
        let state_root = HashValue::new([8; 32]);
        let gas_used = 123;
        let message_bytes = block_permit_message_bytes(
            chain_id,
            ACTIVATION_HEIGHT,
            parent.id(),
            timestamp,
            author,
            difficulty,
            block_accumulator_root,
            state_root,
            gas_used,
            &body_without_permit,
        )
        .unwrap();
        let digest = block_permit_digest(
            chain_id,
            ACTIVATION_HEIGHT,
            parent.id(),
            timestamp,
            author,
            difficulty,
            block_accumulator_root,
            state_root,
            gas_used,
            &body_without_permit,
        )
        .unwrap();
        let raw =
            build_block_permit_raw_transaction(&public_key, author, digest, chain_id).unwrap();
        let permit = raw.sign(&private_key, public_key).unwrap().into_inner();
        let body = BlockBody::new(vec![permit], None);
        let header = BlockHeader::new(
            parent.id(),
            timestamp,
            ACTIVATION_HEIGHT,
            author,
            HashValue::new([6; 32]),
            block_accumulator_root,
            state_root,
            gas_used,
            difficulty,
            body.hash(),
            chain_id,
            0,
            BlockHeaderExtra::default(),
        );
        Fixture {
            policy,
            parent,
            block: Block::new(header, body),
            private_key,
            digest,
            message_bytes,
        }
    }

    fn with_body(block: &Block, body: BlockBody) -> Block {
        let header = block
            .header()
            .as_builder()
            .with_body_hash(body.hash())
            .build();
        Block::new(header, body)
    }

    fn replace_permit(fixture: &Fixture, permit: SignedUserTransaction) -> Block {
        with_body(
            &fixture.block,
            BlockBody::new(vec![permit], fixture.block.body.uncles.clone()),
        )
    }

    #[test]
    fn golden_message_digest_and_envelope() {
        let fixture = fixture();
        assert_eq!(
            hex::encode(&fixture.message_bytes),
            "010a0000000000000020db3f14ca5b723f3c205ee158e9122e330e73ec7805c41014e68634f3f13aa93c87d612000000000011111111111111111111111111111111000000000000000000000000000000000000000000000000000000000000002a2007070707070707070707070707070707070707070707070707070707070707072008080808080808080808080808080808080808080808080808080808080808087b0000000000000020c01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97"
        );
        assert_eq!(
            hex::encode(fixture.digest.to_vec()),
            "13ee92a67a85de49e1747f3960cdb1f2df766afabefc379610f2bec69e508405"
        );
        assert_eq!(
            hex::encode(bcs_ext::to_bytes(&fixture.block.transactions()[0]).unwrap()),
            "e61b780ad0935f3363bc7fce64edbdad000000000000000002000000000000000000000000000000010f5472616e73666572536372697074731d706565725f746f5f706565725f776974685f6d657461646174615f76320107000000000000000000000000000000010353544303535443000310111111111111111111111111111111111000000000000000000000000000000000212013ee92a67a85de49e1747f3960cdb1f2df766afabefc379610f2bec69e508405000000000000000000000000000000000d3078313a3a5354433a3a5354430000000000000000010020ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c405dd92f1408c01b9e3727d0f0ab1dd78b8fa4665f9db32a9eb4e142c20c1894c840c7dff56b7212c50420c56e28954533cdb46afd36effbe935553dcc1f4ad001"
        );
        let script = match fixture.block.transactions()[0].payload() {
            TransactionPayload::ScriptFunction(script) => script,
            _ => unreachable!(),
        };
        assert_eq!(script.args()[2].len(), 33);
        assert_eq!(script.args()[2][0], 32);
        assert_eq!(&script.args()[2][1..], fixture.digest.as_ref());
    }

    #[test]
    fn activation_boundary_and_nonce_exclusion() {
        let fixture = fixture();
        validate_block_permit(
            fixture.policy,
            ChainId::new(1),
            &fixture.parent,
            &fixture.block,
        )
        .unwrap();

        let missing = with_body(&fixture.block, BlockBody::new(vec![], None));
        assert!(
            validate_block_permit(fixture.policy, ChainId::new(1), &fixture.parent, &missing,)
                .is_err()
        );
        assert!(validate_block_permit(
            BlockPermitPolicy::new_for_test(
                POST_ACTIVATION_HEIGHT,
                AuthenticationKey::ed25519(&fixture.private_key.public_key()),
            ),
            ChainId::new(1),
            &fixture.parent,
            &missing,
        )
        .is_ok());

        let changed_header = fixture
            .block
            .header()
            .as_builder()
            .with_nonce(42)
            .with_extra(BlockHeaderExtra::new([9, 8, 7, 6]))
            .build();
        let changed_pow_fields = Block::new(changed_header, fixture.block.body.clone());
        validate_block_permit(
            fixture.policy,
            ChainId::new(1),
            &fixture.parent,
            &changed_pow_fields,
        )
        .unwrap();

        let wrong_chain_header = fixture
            .block
            .header()
            .as_builder()
            .with_chain_id(ChainId::new(2))
            .build();
        assert!(validate_block_permit(
            fixture.policy,
            ChainId::new(1),
            &fixture.parent,
            &Block::new(wrong_chain_header, fixture.block.body.clone()),
        )
        .is_err());
        let auth_key_header = fixture
            .block
            .header()
            .as_builder()
            .with_author_auth_key(Some(AuthenticationKey::ed25519(
                &fixture.private_key.public_key(),
            )))
            .build();
        assert!(validate_block_permit(
            fixture.policy,
            ChainId::new(1),
            &fixture.parent,
            &Block::new(auth_key_header, fixture.block.body.clone()),
        )
        .is_err());
    }

    #[test]
    fn rejects_wrong_key_signature_sender_and_envelope() {
        let fixture = fixture();
        let author = fixture.block.header().author();
        let chain_id = ChainId::new(1);

        let wrong_key = Ed25519PrivateKey::try_from(&[8u8; 32][..]).unwrap();
        let wrong_public_key = wrong_key.public_key();
        let wrong_key_permit =
            build_block_permit_raw_transaction(&wrong_public_key, author, fixture.digest, chain_id)
                .unwrap()
                .sign(&wrong_key, wrong_public_key)
                .unwrap()
                .into_inner();
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &replace_permit(&fixture, wrong_key_permit),
        )
        .is_err());

        let valid_permit = &fixture.block.transactions()[0];
        let (public_key, signature) = match valid_permit.authenticator() {
            TransactionAuthenticator::Ed25519 {
                public_key,
                signature,
            } => (public_key, signature),
            _ => unreachable!(),
        };
        let mut bad_signature_bytes = signature.to_bytes().to_vec();
        bad_signature_bytes[0] ^= 1;
        let bad_signature = Ed25519Signature::try_from(bad_signature_bytes.as_slice()).unwrap();
        let bad_signature_permit = SignedUserTransaction::ed25519(
            valid_permit.raw_txn().clone(),
            public_key,
            bad_signature,
        );
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &replace_permit(&fixture, bad_signature_permit),
        )
        .is_err());

        let raw = valid_permit.raw_txn();
        let wrong_sender_raw = RawUserTransaction::new(
            AccountAddress::new([0x33; 16]),
            raw.sequence_number(),
            raw.payload().clone(),
            raw.max_gas_amount(),
            raw.gas_unit_price(),
            raw.expiration_timestamp_secs(),
            raw.chain_id(),
            raw.gas_token_code(),
        );
        let permit = wrong_sender_raw
            .sign(&fixture.private_key, fixture.private_key.public_key())
            .unwrap()
            .into_inner();
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &replace_permit(&fixture, permit),
        )
        .is_err());

        let wrong_envelope_raw = RawUserTransaction::new(
            raw.sender(),
            raw.sequence_number(),
            raw.payload().clone(),
            1,
            raw.gas_unit_price(),
            raw.expiration_timestamp_secs(),
            raw.chain_id(),
            raw.gas_token_code(),
        );
        let permit = wrong_envelope_raw
            .sign(&fixture.private_key, fixture.private_key.public_key())
            .unwrap()
            .into_inner();
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &replace_permit(&fixture, permit),
        )
        .is_err());

        let wrong_chain_raw = RawUserTransaction::new(
            raw.sender(),
            raw.sequence_number(),
            raw.payload().clone(),
            raw.max_gas_amount(),
            raw.gas_unit_price(),
            raw.expiration_timestamp_secs(),
            ChainId::new(2),
            raw.gas_token_code(),
        );
        let permit = wrong_chain_raw
            .sign(&fixture.private_key, fixture.private_key.public_key())
            .unwrap()
            .into_inner();
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &replace_permit(&fixture, permit),
        )
        .is_err());
    }

    #[test]
    fn rejects_wrong_scheme_digest_and_signed_contents() {
        let fixture = fixture();
        let chain_id = ChainId::new(1);
        let author = fixture.block.header().author();
        let valid_raw = fixture.block.transactions()[0].raw_txn().clone();

        let second_key = Ed25519PrivateKey::try_from(&[9u8; 32][..]).unwrap();
        let multi_public_key = MultiEd25519PublicKey::new(
            vec![fixture.private_key.public_key(), second_key.public_key()],
            1,
        )
        .unwrap();
        let multi_signature =
            MultiEd25519Signature::new(vec![(fixture.private_key.sign(&valid_raw), 0)]).unwrap();
        let multi_permit =
            SignedUserTransaction::multi_ed25519(valid_raw, multi_public_key, multi_signature);
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &replace_permit(&fixture, multi_permit),
        )
        .is_err());

        let public_key = fixture.private_key.public_key();
        let bad_digest_raw = build_block_permit_raw_transaction(
            &public_key,
            author,
            HashValue::new([0x44; 32]),
            chain_id,
        )
        .unwrap();
        let bad_digest_permit = bad_digest_raw
            .sign(&fixture.private_key, public_key)
            .unwrap()
            .into_inner();
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &replace_permit(&fixture, bad_digest_permit),
        )
        .is_err());

        let changed_header = fixture
            .block
            .header()
            .as_builder()
            .with_timestamp(fixture.block.header().timestamp() + 1)
            .build();
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &Block::new(changed_header, fixture.block.body.clone()),
        )
        .is_err());

        let duplicated_ordinary = fixture.block.transactions()[0].clone();
        let changed_body = BlockBody::new(
            vec![duplicated_ordinary, fixture.block.transactions()[0].clone()],
            None,
        );
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &with_body(&fixture.block, changed_body),
        )
        .is_err());

        let changed_uncles = BlockBody::new(
            vec![fixture.block.transactions()[0].clone()],
            Some(vec![fixture.parent.clone()]),
        );
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &with_body(&fixture.block, changed_uncles),
        )
        .is_err());

        let permit_not_final = BlockBody::new(
            vec![
                fixture.block.transactions()[0].clone(),
                SignedUserTransaction::mock(),
            ],
            None,
        );
        assert!(validate_block_permit(
            fixture.policy,
            chain_id,
            &fixture.parent,
            &with_body(&fixture.block, permit_not_final),
        )
        .is_err());
    }

    #[test]
    fn generation_precedes_difficulty_only_for_configured_release_policy() {
        let fixture = fixture();
        assert!(
            validated_chain_quality(fixture.policy, ACTIVATION_HEIGHT, U256::from(1u64),)
                > validated_chain_quality(fixture.policy, PRE_ACTIVATION_HEIGHT, U256::max_value(),)
        );
        assert!(
            advertised_sync_rank(fixture.policy, ACTIVATION_HEIGHT, U256::from(1u64),)
                > advertised_sync_rank(fixture.policy, PRE_ACTIVATION_HEIGHT, U256::max_value(),)
        );
        assert!(
            validated_chain_quality(
                BlockPermitPolicy::disabled(),
                ACTIVATION_HEIGHT,
                U256::from(1u64),
            ) < validated_chain_quality(
                BlockPermitPolicy::disabled(),
                PRE_ACTIVATION_HEIGHT,
                U256::from(2u64),
            )
        );
    }

    #[test]
    fn release_height_and_key_must_be_configured_together() {
        let fixture = fixture();
        let dummy_key = AuthenticationKey::new(AuthenticationKey::DUMMY_KEY);
        assert!(BlockPermitPolicy::disabled().release_inputs_consistent());
        assert!(!BlockPermitPolicy {
            trusted_chain_id: Some(ChainId::new(1)),
            activation_height: ACTIVATION_HEIGHT,
            authentication_key: dummy_key,
        }
        .release_inputs_consistent());
        assert!(!BlockPermitPolicy {
            trusted_chain_id: Some(ChainId::new(1)),
            activation_height: BlockNumber::MAX,
            authentication_key: AuthenticationKey::ed25519(&fixture.private_key.public_key()),
        }
        .release_inputs_consistent());
        assert!(fixture.policy.release_inputs_consistent());
    }

    #[test]
    fn release_network_inputs_are_fixed_and_distinct() {
        assert_eq!(HALLEY_BLOCK_PERMIT_HEIGHT, 2_894_400);
        assert_eq!(BARNARD_BLOCK_PERMIT_HEIGHT, 19_667_300);
        assert_eq!(MAINNET_BLOCK_PERMIT_HEIGHT, 32_165_300);

        let policies = [
            BlockPermitPolicy::for_trusted_builtin(ChainId::new(253)),
            BlockPermitPolicy::for_trusted_builtin(ChainId::new(251)),
            BlockPermitPolicy::for_trusted_builtin(ChainId::new(1)),
        ];
        assert_eq!(policies[0].activation_height(), HALLEY_BLOCK_PERMIT_HEIGHT);
        assert_eq!(policies[1].activation_height(), BARNARD_BLOCK_PERMIT_HEIGHT);
        assert_eq!(policies[2].activation_height(), MAINNET_BLOCK_PERMIT_HEIGHT);
        assert!(policies.iter().all(|policy| policy.release_configured()));

        let keys = policies
            .iter()
            .map(|policy| {
                policy
                    .authentication_key(policy.activation_height())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[0], keys[2]);
        assert_ne!(keys[1], keys[2]);
    }
}
