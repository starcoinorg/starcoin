use super::{tree::*, *};
use crate::consensusdb::schemadb::{ReachabilityStore, ReachabilityStoreReader};
use crate::process_key_already_error;
use crate::types::{interval::Interval, perf};
use starcoin_crypto::{HashValue as Hash, HashValue};

/// Init the reachability store to match the state required by the algorithmic layer.
/// The function first checks the store for possibly being initialized already.
pub fn init(store: &mut (impl ReachabilityStore + ?Sized), origin: HashValue) -> Result<()> {
    init_with_params(store, origin, Interval::maximal())
}

pub fn init_for_test(
    store: &mut (impl ReachabilityStore + ?Sized),
    origin: HashValue,
    capacity: Interval,
) -> Result<()> {
    init_with_params(store, origin, capacity)
}

pub(super) fn init_with_params(
    store: &mut (impl ReachabilityStore + ?Sized),
    origin: Hash,
    capacity: Interval,
) -> Result<()> {
    if store.has(origin)? {
        return Ok(());
    }
    store.init(origin, capacity)?;
    Ok(())
}

type HashIterator<'a> = &'a mut dyn Iterator<Item = Hash>;

/// Add a block to the DAG reachability data structures and persist using the provided `store`.
pub fn add_block(
    store: &mut (impl ReachabilityStore + ?Sized),
    new_block: Hash,
    selected_parent: Hash,
    mergeset_iterator: HashIterator,
) -> Result<()> {
    add_block_with_params(
        store,
        new_block,
        selected_parent,
        mergeset_iterator,
        None,
        None,
    )
}

fn add_block_with_params(
    store: &mut (impl ReachabilityStore + ?Sized),
    new_block: Hash,
    selected_parent: Hash,
    mergeset_iterator: HashIterator,
    reindex_depth: Option<u64>,
    reindex_slack: Option<u64>,
) -> Result<()> {
    add_tree_block(
        store,
        new_block,
        selected_parent,
        reindex_depth.unwrap_or(perf::DEFAULT_REINDEX_DEPTH),
        reindex_slack.unwrap_or(perf::DEFAULT_REINDEX_SLACK),
    )?;
    add_dag_block(store, new_block, mergeset_iterator)?;
    Ok(())
}

fn add_dag_block(
    store: &mut (impl ReachabilityStore + ?Sized),
    new_block: Hash,
    mergeset_iterator: HashIterator,
) -> Result<()> {
    // Update the future covering set for blocks in the mergeset
    for merged_block in mergeset_iterator {
        insert_to_future_covering_set(store, merged_block, new_block)?;
    }
    Ok(())
}

fn insert_to_future_covering_set(
    store: &mut (impl ReachabilityStore + ?Sized),
    merged_block: Hash,
    new_block: Hash,
) -> Result<()> {
    let result = binary_search_descendant(
        store,
        store.get_future_covering_set(merged_block)?.as_slice(),
        new_block,
    );
    match result {
        Ok(search_output) => {
            match search_output {
                // We expect the query to not succeed, and to only return the correct insertion index.
                // The existences of a `future covering item` (`FCI`) which is a chain ancestor of `new_block`
                // contradicts `merged_block ∈ mergeset(new_block)`. Similarly, the existence of an FCI
                // which `new_block` is a chain ancestor of, contradicts processing order.
                SearchOutput::Found(_, _) => Err(ReachabilityError::DataInconsistency),
                SearchOutput::NotFound(i) => {
                    process_key_already_error(store.insert_future_covering_item(
                        merged_block,
                        new_block,
                        i,
                    ))?;
                    Ok(())
                }
            }
        }
        Err(ReachabilityError::HashesNotOrdered) => {
            let future_covering_set = store.get_future_covering_set(merged_block)?;
            if future_covering_set.contains(&new_block) {
                Ok(())
            } else {
                Err(ReachabilityError::HashesNotOrdered)
            }
        }
        Err(e) => Err(e),
    }
}

/// Hint to the reachability algorithm that `hint` is a candidate to become
/// the `virtual selected parent` (`VSP`). This might affect internal reachability heuristics such
/// as moving the reindex point. The consensus runtime is expected to call this function
/// for a new header selected tip which is `header only` / `pending UTXO verification`, or for a completely resolved `VSP`.
pub fn hint_virtual_selected_parent(
    store: &mut (impl ReachabilityStore + ?Sized),
    hint: Hash,
) -> Result<()> {
    try_advancing_reindex_root(
        store,
        hint,
        perf::DEFAULT_REINDEX_DEPTH,
        perf::DEFAULT_REINDEX_SLACK,
    )
}

/// Checks if the `this` block is a strict chain ancestor of the `queried` block (aka `this ∈ chain(queried)`).
/// Note that this results in `false` if `this == queried`
pub fn is_strict_chain_ancestor_of(
    store: &(impl ReachabilityStoreReader + ?Sized),
    this: Hash,
    queried: Hash,
) -> Result<bool> {
    Ok(store
        .get_interval(this)?
        .strictly_contains(store.get_interval(queried)?))
}

/// Checks if `this` block is a chain ancestor of `queried` block (aka `this ∈ chain(queried) ∪ {queried}`).
/// Note that we use the graph theory convention here which defines that a block is also an ancestor of itself.
pub fn is_chain_ancestor_of(
    store: &(impl ReachabilityStoreReader + ?Sized),
    this: Hash,
    queried: Hash,
) -> Result<bool> {
    Ok(store
        .get_interval(this)?
        .contains(store.get_interval(queried)?))
}

/// Returns true if `this` is a DAG ancestor of `queried` (aka `queried ∈ future(this) ∪ {this}`).
/// Note: this method will return true if `this == queried`.
/// The complexity of this method is O(log(|future_covering_set(this)|))
pub fn is_dag_ancestor_of(
    store: &(impl ReachabilityStoreReader + ?Sized),
    this: Hash,
    queried: Hash,
) -> Result<bool> {
    // First, check if `this` is a chain ancestor of queried
    if is_chain_ancestor_of(store, this, queried)? {
        return Ok(true);
    }
    // Otherwise, use previously registered future blocks to complete the
    // DAG reachability test
    match binary_search_descendant(
        store,
        store.get_future_covering_set(this)?.as_slice(),
        queried,
    )? {
        SearchOutput::Found(_, _) => Ok(true),
        SearchOutput::NotFound(_) => Ok(false),
    }
}

/// Finds the child of `ancestor` which is also a chain ancestor of `descendant`.
pub fn get_next_chain_ancestor(
    store: &(impl ReachabilityStoreReader + ?Sized),
    descendant: Hash,
    ancestor: Hash,
) -> Result<Hash> {
    if descendant == ancestor {
        // The next ancestor does not exist
        return Err(ReachabilityError::BadQuery);
    }
    if !is_strict_chain_ancestor_of(store, ancestor, descendant)? {
        // `ancestor` isn't actually a chain ancestor of `descendant`, so by def
        // we cannot find the next ancestor as well
        return Err(ReachabilityError::BadQuery);
    }

    get_next_chain_ancestor_unchecked(store, descendant, ancestor)
}

/// Note: it is important to keep the unchecked version for internal module use,
/// since in some scenarios during reindexing `descendant` might have a modified
/// interval which was not propagated yet.
pub(super) fn get_next_chain_ancestor_unchecked(
    store: &(impl ReachabilityStoreReader + ?Sized),
    descendant: Hash,
    ancestor: Hash,
) -> Result<Hash> {
    match binary_search_descendant(store, store.get_children(ancestor)?.as_slice(), descendant)? {
        SearchOutput::Found(hash, _) => Ok(hash),
        SearchOutput::NotFound(_) => Err(ReachabilityError::BadQuery),
    }
}

enum SearchOutput {
    NotFound(usize), // `usize` is the position to insert at
    Found(Hash, usize),
}

fn binary_search_descendant(
    store: &(impl ReachabilityStoreReader + ?Sized),
    ordered_hashes: &[Hash],
    descendant: Hash,
) -> Result<SearchOutput> {
    if cfg!(debug_assertions) {
        // This is a linearly expensive assertion, keep it debug only
        assert_hashes_ordered(store, ordered_hashes)?;
    }

    // `Interval::end` represents the unique number allocated to this block
    let point = store.get_interval(descendant)?.end;

    // We use an `unwrap` here since otherwise we need to implement `binary_search`
    // ourselves, which is not worth the effort given that this would be an unrecoverable
    // error anyhow
    match ordered_hashes.binary_search_by_key(&point, |c| store.get_interval(*c).unwrap().start) {
        Ok(i) => Ok(SearchOutput::Found(ordered_hashes[i], i)),
        Err(i) => {
            // `i` is where `point` was expected (i.e., point < ordered_hashes[i].interval.start),
            // so we expect `ordered_hashes[i - 1].interval` to be the only candidate to contain `point`
            if i > 0
                && is_chain_ancestor_of(
                    store,
                    ordered_hashes[i.checked_sub(1).unwrap()],
                    descendant,
                )?
            {
                Ok(SearchOutput::Found(
                    ordered_hashes[i.checked_sub(1).unwrap()],
                    i.checked_sub(1).unwrap(),
                ))
            } else {
                Ok(SearchOutput::NotFound(i))
            }
        }
    }
}

fn assert_hashes_ordered(
    store: &(impl ReachabilityStoreReader + ?Sized),
    ordered_hashes: &[Hash],
) -> Result<()> {
    let intervals: Vec<Interval> = ordered_hashes
        .iter()
        .cloned()
        .map(|c| store.get_interval(c).unwrap())
        .collect();
    if intervals
        .as_slice()
        .windows(2)
        .all(|w| w[0].end < w[1].start)
    {
        return Ok(());
    }
    Err(ReachabilityError::HashesNotOrdered)
    // debug_assert!(intervals
    //     .as_slice()
    //     .windows(2)
    //     .all(|w| w[0].end < w[1].start))
}

#[cfg(test)]
mod tests {
    use super::{super::tests::*, *};
    use crate::consensusdb::schemadb::MemoryReachabilityStore;
    use starcoin_types::blockhash::ORIGIN;

    #[test]
    fn test_add_tree_blocks() {
        // Arrange
        let mut store = MemoryReachabilityStore::new();
        // Act
        let root: Hash = 1.into();
        TreeBuilder::new(&mut store)
            .init_with_params(root, Interval::new(1, 15))
            .add_block(2.into(), root)
            .add_block(3.into(), 2.into())
            .add_block(4.into(), 2.into())
            .add_block(5.into(), 3.into())
            .add_block(6.into(), 5.into())
            .add_block(7.into(), 1.into())
            .add_block(8.into(), 6.into())
            .add_block(9.into(), 6.into())
            .add_block(10.into(), 6.into())
            .add_block(11.into(), 6.into());
        // Assert
        store.validate_intervals(root).unwrap();
    }

    #[test]
    fn test_add_early_blocks() {
        // Arrange
        let mut store = MemoryReachabilityStore::new();

        // Act
        let root: Hash = Hash::from_u64(1);
        let mut builder = TreeBuilder::new_with_params(&mut store, 2, 5);
        builder.init_with_params(root, Interval::maximal());
        for i in 2u64..100 {
            builder.add_block(Hash::from_u64(i), Hash::from_u64(i / 2));
        }

        // Should trigger an earlier than reindex root allocation
        builder.add_block(Hash::from_u64(100), Hash::from_u64(2));
        store.validate_intervals(root).unwrap();
    }

    #[test]
    fn test_add_dag_blocks() {
        // Arrange
        let mut store = MemoryReachabilityStore::new();
        let origin_hash = Hash::new(ORIGIN);
        // Act
        DagBuilder::new(&mut store)
            .init(origin_hash)
            .add_block(DagBlock::new(1.into(), vec![origin_hash]))
            .add_block(DagBlock::new(2.into(), vec![1.into()]))
            .add_block(DagBlock::new(3.into(), vec![1.into()]))
            .add_block(DagBlock::new(4.into(), vec![2.into(), 3.into()]))
            .add_block(DagBlock::new(5.into(), vec![4.into()]))
            .add_block(DagBlock::new(6.into(), vec![1.into()]))
            .add_block(DagBlock::new(7.into(), vec![5.into(), 6.into()]))
            .add_block(DagBlock::new(8.into(), vec![1.into()]))
            .add_block(DagBlock::new(9.into(), vec![1.into()]))
            .add_block(DagBlock::new(10.into(), vec![7.into(), 8.into(), 9.into()]))
            .add_block(DagBlock::new(11.into(), vec![1.into()]))
            .add_block(DagBlock::new(12.into(), vec![11.into(), 10.into()]));

        // Assert intervals
        store.validate_intervals(origin_hash).unwrap();

        // Assert genesis
        for i in 2u64..=12 {
            assert!(store.in_past_of(1, i));
        }

        // Assert some futures
        assert!(store.in_past_of(2, 4));
        assert!(store.in_past_of(2, 5));
        assert!(store.in_past_of(2, 7));
        assert!(store.in_past_of(5, 10));
        assert!(store.in_past_of(6, 10));
        assert!(store.in_past_of(10, 12));
        assert!(store.in_past_of(11, 12));

        // Assert some anticones
        assert!(store.are_anticone(2, 3));
        assert!(store.are_anticone(2, 6));
        assert!(store.are_anticone(3, 6));
        assert!(store.are_anticone(5, 6));
        assert!(store.are_anticone(3, 8));
        assert!(store.are_anticone(11, 2));
        assert!(store.are_anticone(11, 4));
        assert!(store.are_anticone(11, 6));
        assert!(store.are_anticone(11, 9));
    }
}
