// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Block-production check that DAO vote tallies stay backed by staked tokens.
//!
//! While a proposal is open for voting, every change to its for + against votes must match the
//! change in the stake of the `Dao::Vote` resources that point at it. Casting, flipping and
//! revoking all keep the two equal; the block producer leaves out any txn that breaks the
//! balance. This is a production policy, not a consensus rule.

use anyhow::{format_err, Result};
use starcoin_state_api::ChainStateReader;
use starcoin_vm_types::access_path::{AccessPath, DataPath};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::language_storage::{StructTag, TypeTag};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::write_set::{WriteOp, WriteSet};
use std::collections::BTreeMap;

/// One proposal: its token, proposer and id. Ids are unique per token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProposalKey {
    token: TypeTag,
    proposer: AccountAddress,
    id: u64,
}

/// `Dao::Vote<TokenT>`: proposer, id, stake (`Token { value: u128 }`), agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VoteHead {
    proposer: AccountAddress,
    id: u64,
    stake: u128,
}

/// The fixed leading fields of `Dao::Proposal<TokenT, ActionT>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProposalHead {
    id: u64,
    end_time_ms: u64,
    votes: u128,
}

fn dao_struct<'a>(key: &'a StateKey, name: &str) -> Option<(AccountAddress, &'a StructTag)> {
    match key {
        StateKey::AccessPath(AccessPath {
            address,
            path: DataPath::Resource(tag),
        }) if tag.address == genesis_address()
            && tag.module.as_str() == "Dao"
            && tag.name.as_str() == name =>
        {
            Some((*address, tag))
        }
        _ => None,
    }
}

fn take<const N: usize>(bytes: &[u8], at: usize) -> Result<[u8; N]> {
    bytes
        .get(at..at + N)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| format_err!("resource too short: {} bytes", bytes.len()))
}

fn parse_vote(bytes: &[u8]) -> Result<VoteHead> {
    let len = AccountAddress::LENGTH;
    Ok(VoteHead {
        proposer: AccountAddress::new(take::<{ AccountAddress::LENGTH }>(bytes, 0)?),
        id: u64::from_le_bytes(take::<8>(bytes, len)?),
        stake: u128::from_le_bytes(take::<16>(bytes, len + 8)?),
    })
}

/// id: u64, proposer: address, start_time: u64, end_time: u64, for_votes: u128,
/// against_votes: u128, ...
fn parse_proposal(bytes: &[u8]) -> Result<ProposalHead> {
    let end_at = 8 + AccountAddress::LENGTH + 8;
    let for_at = end_at + 8;
    let for_votes = u128::from_le_bytes(take::<16>(bytes, for_at)?);
    let against_votes = u128::from_le_bytes(take::<16>(bytes, for_at + 16)?);
    Ok(ProposalHead {
        id: u64::from_le_bytes(take::<8>(bytes, 0)?),
        end_time_ms: u64::from_le_bytes(take::<8>(bytes, end_at)?),
        votes: for_votes.saturating_add(against_votes),
    })
}

/// One proposal's view across the txn.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Ledger {
    before: Option<ProposalHead>,
    after: Option<ProposalHead>,
    /// Stake added to minus stake removed from votes pointing at the proposal.
    stake_delta: i128,
}

/// Whether the tally moved differently from the stake behind it while voting was open.
fn unbacked(ledger: &Ledger, now_ms: u64) -> bool {
    let open = match ledger.before.or(ledger.after) {
        Some(p) => now_ms <= p.end_time_ms,
        // The proposal is gone: unstaking from it is what the framework allows.
        None => return false,
    };
    let votes = |p: Option<ProposalHead>| p.map_or(0, |p| p.votes as i128);
    open && votes(ledger.after) - votes(ledger.before) != ledger.stake_delta
}

fn find_proposal(state: &dyn ChainStateReader, key: &ProposalKey) -> Result<Option<ProposalHead>> {
    let account = match state.get_account_state_set(&key.proposer)? {
        Some(account) => account,
        None => return Ok(None),
    };
    for (tag_bytes, value) in account.resource_set().into_iter().flat_map(|s| s.iter()) {
        let tag: StructTag = bcs_ext::from_bytes(tag_bytes)?;
        if tag.address == genesis_address()
            && tag.module.as_str() == "Dao"
            && tag.name.as_str() == "Proposal"
            && tag.type_params.first() == Some(&key.token)
        {
            let head = parse_proposal(value)?;
            if head.id == key.id {
                return Ok(Some(head));
            }
        }
    }
    Ok(None)
}

/// The proposals whose tally the txn changes without the matching stake, given `state` before
/// the txn and the txn's `write_set`.
pub(crate) fn unbacked_tallies(
    state: &dyn ChainStateReader,
    write_set: &WriteSet,
    now_ms: u64,
) -> Result<Vec<(AccountAddress, u64)>> {
    let mut ledgers: BTreeMap<ProposalKey, Ledger> = BTreeMap::new();
    for (key, op) in write_set.iter() {
        if let Some((_, tag)) = dao_struct(key, "Vote") {
            let token = match tag.type_params.first() {
                Some(token) => token.clone(),
                None => continue,
            };
            let mut book = |v: VoteHead, sign: i128| {
                let entry = ledgers
                    .entry(ProposalKey {
                        token: token.clone(),
                        proposer: v.proposer,
                        id: v.id,
                    })
                    .or_default();
                entry.stake_delta += sign * v.stake as i128;
            };
            if let Some(bytes) = state.get_state_value(key)? {
                book(parse_vote(&bytes)?, -1);
            }
            if let WriteOp::Value(bytes) = op {
                book(parse_vote(bytes)?, 1);
            }
        } else if let Some((proposer, tag)) = dao_struct(key, "Proposal") {
            let token = match tag.type_params.first() {
                Some(token) => token.clone(),
                None => continue,
            };
            let before = state
                .get_state_value(key)?
                .map(|b| parse_proposal(&b))
                .transpose()?;
            let after = match op {
                WriteOp::Value(bytes) => Some(parse_proposal(bytes)?),
                WriteOp::Deletion => None,
            };
            let id = match before.or(after) {
                Some(p) => p.id,
                None => continue,
            };
            let entry = ledgers
                .entry(ProposalKey {
                    token,
                    proposer,
                    id,
                })
                .or_default();
            entry.before = before;
            entry.after = after;
        }
    }
    let mut out = vec![];
    for (key, mut ledger) in ledgers {
        if ledger.before.is_none() && ledger.after.is_none() {
            // Only votes moved; the proposal itself is unchanged by the txn.
            let current = find_proposal(state, &key)?;
            ledger.before = current;
            ledger.after = current;
        }
        if unbacked(&ledger, now_ms) {
            out.push((key.proposer, key.id));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proposal(votes: u128) -> Option<ProposalHead> {
        Some(ProposalHead {
            id: 7,
            end_time_ms: 1_000,
            votes,
        })
    }

    fn ledger(before: u128, after: u128, stake_delta: i128) -> Ledger {
        Ledger {
            before: proposal(before),
            after: proposal(after),
            stake_delta,
        }
    }

    #[test]
    fn honest_vote_changes_keep_the_balance() {
        // cast 50, revoke 30, flip (no change in sum or stake)
        assert!(!unbacked(&ledger(100, 150, 50), 999));
        assert!(!unbacked(&ledger(100, 70, -30), 999));
        assert!(!unbacked(&ledger(100, 100, 0), 999));
        // a new proposal starts at zero with no stake
        let created = Ledger {
            before: None,
            after: proposal(0),
            stake_delta: 0,
        };
        assert!(!unbacked(&created, 0));
    }

    #[test]
    fn stake_leaving_a_counted_tally_is_unbacked() {
        // stake taken back while the tally stays
        assert!(unbacked(&ledger(100, 100, -100), 999));
        // stake taken back and re-cast in one txn: stake flat, tally up
        assert!(unbacked(&ledger(100, 200, 0), 1_000));
    }

    #[test]
    fn after_voting_or_without_the_proposal_anything_goes() {
        assert!(!unbacked(&ledger(100, 100, -100), 1_001));
        let gone = Ledger {
            before: None,
            after: None,
            stake_delta: -100,
        };
        assert!(!unbacked(&gone, 0));
    }

    #[test]
    fn parses_the_framework_layouts() {
        let mut v = AccountAddress::ONE.to_vec();
        v.extend(7u64.to_le_bytes());
        v.extend(5u128.to_le_bytes());
        v.push(1);
        let vote = parse_vote(&v).unwrap();
        assert_eq!(
            (vote.proposer, vote.id, vote.stake),
            (AccountAddress::ONE, 7, 5)
        );

        let mut p = 7u64.to_le_bytes().to_vec();
        p.extend(AccountAddress::ONE.to_vec());
        p.extend(10u64.to_le_bytes());
        p.extend(1_000u64.to_le_bytes());
        p.extend(60u128.to_le_bytes());
        p.extend(40u128.to_le_bytes());
        p.extend([0u8; 40]);
        assert_eq!(parse_proposal(&p).unwrap(), proposal(100).unwrap());
        assert!(parse_proposal(&p[..20]).is_err());
    }
}

#[cfg(test)]
mod chain_tests {
    use super::*;
    use starcoin_crypto::HashValue;
    use starcoin_executor::execute_transactions;
    use starcoin_types::account::Account;
    use starcoin_types::block_metadata::BlockMetadata;
    use starcoin_types::transaction::{ScriptFunction, Transaction, TransactionStatus};
    use starcoin_vm_types::account_config::{core_code_address, stc_type_tag};
    use starcoin_vm_types::identifier::Identifier;
    use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
    use starcoin_vm_types::transaction::TransactionPayload;
    use starcoin_vm_types::vm_status::KeptVMStatus;
    use test_helper::dao::{
        execute_cast_vote, execute_create_account, on_chain_config_type_tag,
        transaction_timeout_type_tag, vote_txn_timeout_script, voting_delay,
    };
    use test_helper::executor::{
        account_execute_should_success, blockmeta_execute, build_signed_txn, current_block_number,
        prepare_genesis,
    };

    fn vote_call(function: &str, action: TypeTag, proposer: AccountAddress) -> TransactionPayload {
        TransactionPayload::ScriptFunction(ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("DaoVoteScripts").unwrap(),
            ),
            Identifier::new(function).unwrap(),
            vec![stc_type_tag(), action],
            vec![
                bcs_ext::to_bytes(&proposer).unwrap(),
                bcs_ext::to_bytes(&0u64).unwrap(),
            ],
        ))
    }

    #[stest::test]
    fn unbalanced_tally_changes_are_flagged_and_revoke_passes() {
        let (state, net) = prepare_genesis();
        let alice = Account::new();
        let day = 24 * 3600 * 1000;
        let number = current_block_number(&state) + 1;
        let ts = net.time_service().now_millis() + day * number;
        execute_create_account(
            &state,
            &net,
            &alice,
            net.genesis_config().pre_mine_amount,
            number,
            ts,
        )
        .unwrap();
        let number = number + 1;
        let ts = ts + day;
        blockmeta_execute(
            &state,
            BlockMetadata::new(
                HashValue::zero(),
                ts,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                number,
                net.chain_id(),
                0,
            ),
        )
        .unwrap();
        account_execute_should_success(
            &alice,
            &state,
            TransactionPayload::ScriptFunction(vote_txn_timeout_script(&net, 240)),
        )
        .unwrap();
        let action = on_chain_config_type_tag(transaction_timeout_type_tag());
        let voting_ts = ts + voting_delay(&state, stc_type_tag()) + 10_000;
        execute_cast_vote(&net, &state, &alice, &action, number + 1, voting_ts, 0).unwrap();

        let run = |payload| {
            let txn = build_signed_txn(*alice.address(), alice.private_key(), &state, payload);
            execute_transactions(&state, vec![Transaction::UserTransaction(txn)], None)
                .unwrap()
                .pop()
                .unwrap()
        };

        // A txn that moves stake without the tally is flagged.
        let unbalanced = run(vote_call("unstake_vote", TypeTag::U64, *alice.address()));
        assert_eq!(
            unbalanced.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        assert_eq!(
            unbacked_tallies(&state, unbalanced.write_set(), voting_ts).unwrap(),
            vec![(*alice.address(), 0)]
        );

        // An unstake the framework refuses changes nothing.
        let refused = run(vote_call("unstake_vote", action.clone(), *alice.address()));
        assert_ne!(
            refused.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        assert!(unbacked_tallies(&state, refused.write_set(), voting_ts)
            .unwrap()
            .is_empty());

        // Stake flat while the tally moves is flagged as well.
        let recast = test_helper::executor::compile_script(
            r#"
            script {
                use 0x1::Dao;
                use 0x1::STC::STC;
                use 0x1::OnChainConfigDao::OnChainConfigUpdate;
                use 0x1::TransactionTimeoutConfig::TransactionTimeoutConfig;
                fun main(account: signer, proposer: address) {
                    let stake = Dao::unstake_votes<STC, u64>(&account, proposer, 0);
                    Dao::cast_vote<STC, OnChainConfigUpdate<TransactionTimeoutConfig>>(
                        &account, proposer, 0, stake, true);
                }
            }
            "#,
        )
        .unwrap();
        let recast = run(TransactionPayload::Script(
            starcoin_vm_types::transaction::Script::new(
                recast,
                vec![],
                vec![bcs_ext::to_bytes(alice.address()).unwrap()],
            ),
        ));
        assert_eq!(
            recast.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        assert_eq!(
            unbacked_tallies(&state, recast.write_set(), voting_ts).unwrap(),
            vec![(*alice.address(), 0)]
        );

        // Revoking during voting takes the vote off the tally and passes the guard.
        let revoke = run(vote_call("revoke_vote", action, *alice.address()));
        assert_eq!(
            revoke.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        assert!(unbacked_tallies(&state, revoke.write_set(), voting_ts)
            .unwrap()
            .is_empty());
    }
}
