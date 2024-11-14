// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Ok, Result};
use starcoin_crypto::HashValue as Hash;
use starcoin_dag::{
    blockdag::{BlockDAG, MineNewDagBlockInfo},
    consensusdb::{
        consenses_state::{DagState, DagStateReader, DagStateStore},
        schemadb::{
            DbReachabilityStore, GhostdagStoreReader, ReachabilityStore, ReachabilityStoreReader,
            RelationsStore, RelationsStoreReader,
        },
    },
    reachability::{inquirer, ReachabilityError},
    types::{ghostdata::GhostdagData, interval::Interval},
};
use starcoin_logger::prelude::debug;
use starcoin_types::{
    block::{BlockHeader, BlockHeaderBuilder, BlockNumber},
    blockhash::{BlockHashMap, HashKTypeMap, KType},
    consensus_header::ConsensusHeader,
    U256,
};

use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
    vec,
};

#[test]
fn test_dag_commit() -> Result<()> {
    let mut dag = BlockDAG::create_for_testing().unwrap();
    let genesis = BlockHeader::dag_genesis_random(0)
        .as_builder()
        .with_difficulty(0.into())
        .build();

    let mut parents_hash = vec![genesis.id()];
    let origin = dag.init_with_genesis(genesis.clone())?;

    for _ in 0..10 {
        let header_builder = BlockHeaderBuilder::random();
        let header = header_builder
            .with_parents_hash(parents_hash.clone())
            .build();
        parents_hash = vec![header.id()];
        dag.commit(header.to_owned(), origin)?;
        let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        println!("{:?},{:?}", header, ghostdata);
    }

    Ok(())
}

#[test]
fn test_dag_1() -> Result<()> {
    let genesis = BlockHeader::dag_genesis_random(0)
        .as_builder()
        .with_difficulty(0.into())
        .build();
    let block1 = BlockHeaderBuilder::random()
        .with_difficulty(1.into())
        .with_parents_hash(vec![genesis.id()])
        .build();
    let block2 = BlockHeaderBuilder::random()
        .with_difficulty(2.into())
        .with_parents_hash(vec![genesis.id()])
        .build();
    let block3_1 = BlockHeaderBuilder::random()
        .with_difficulty(1.into())
        .with_parents_hash(vec![genesis.id()])
        .build();
    let block3 = BlockHeaderBuilder::random()
        .with_difficulty(3.into())
        .with_parents_hash(vec![block3_1.id()])
        .build();
    let block4 = BlockHeaderBuilder::random()
        .with_difficulty(4.into())
        .with_parents_hash(vec![block1.id(), block2.id()])
        .build();
    let block5 = BlockHeaderBuilder::random()
        .with_difficulty(4.into())
        .with_parents_hash(vec![block2.id(), block3.id()])
        .build();
    let block6 = BlockHeaderBuilder::random()
        .with_difficulty(5.into())
        .with_parents_hash(vec![block4.id(), block5.id()])
        .build();
    let mut latest_id = block6.id();
    let genesis_id = genesis.id();
    let mut dag = BlockDAG::create_for_testing().unwrap();
    let expect_selected_parented = [block5.id(), block3.id(), block3_1.id(), genesis_id];
    let origin = dag.init_with_genesis(genesis.clone()).unwrap();

    dag.commit(block1, origin)?;
    dag.commit(block2, origin)?;
    dag.commit(block3_1, origin)?;
    dag.commit(block3, origin)?;
    dag.commit(block4, origin)?;
    dag.commit(block5, origin)?;
    dag.commit(block6, origin)?;
    let mut count = 0;
    while latest_id != genesis_id && count < 4 {
        let ghostdata = dag
            .ghostdata_by_hash(latest_id)?
            .ok_or_else(|| format_err!("Failed to get ghostdata"))?;
        latest_id = ghostdata.selected_parent;
        assert_eq!(expect_selected_parented[count], latest_id);
        count += 1;
    }

    Ok(())
}

#[tokio::test]
async fn test_with_spawn() {
    use starcoin_types::block::{BlockHeader, BlockHeaderBuilder};
    let genesis = BlockHeader::dag_genesis_random(0)
        .as_builder()
        .with_difficulty(0.into())
        .build();
    let block1 = BlockHeaderBuilder::random()
        .with_difficulty(1.into())
        .with_parents_hash(vec![genesis.id()])
        .build();
    let block2 = BlockHeaderBuilder::random()
        .with_difficulty(2.into())
        .with_parents_hash(vec![genesis.id()])
        .build();
    let mut dag = BlockDAG::create_for_testing().unwrap();
    let real_origin = dag.init_with_genesis(genesis.clone()).unwrap();
    dag.commit(block1.clone(), real_origin).unwrap();
    dag.commit(block2.clone(), real_origin).unwrap();
    let block3 = BlockHeaderBuilder::random()
        .with_difficulty(3.into())
        .with_parents_hash(vec![block1.id(), block2.id()])
        .build();
    let mut handles = vec![];
    for i in 1..100 {
        let mut dag_clone = dag.clone();
        let block_clone = block3.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let mut count = 10;
            loop {
                match dag_clone.commit(block_clone.clone(), real_origin) {
                    std::result::Result::Ok(_) => break,
                    Err(e) => {
                        debug!("failed to commit error: {:?}, i: {:?}", e, i);
                        if dag_clone.has_dag_block(block_clone.id()).unwrap() {
                            break;
                        }
                        count -= 1;
                        if count < 0 {
                            panic!("failed to commit block because: {:?}", e);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }
    let mut child = dag.get_children(block1.id()).unwrap();
    assert_eq!(child.pop().unwrap(), block3.id());
    assert_eq!(child.len(), 0);
}

#[test]
fn test_write_asynchronization() -> anyhow::Result<()> {
    let mut dag = BlockDAG::create_for_testing()?;
    let genesis = BlockHeader::dag_genesis_random(0)
        .as_builder()
        .with_difficulty(0.into())
        .build();
    let _real_origin = dag.init_with_genesis(genesis.clone())?;

    let parent = BlockHeaderBuilder::random()
        .with_difficulty(0.into())
        .build();

    let one = BlockHeaderBuilder::random()
        .with_difficulty(0.into())
        .with_parent_hash(parent.id())
        .with_parents_hash(vec![parent.id()])
        .build();

    let two = BlockHeaderBuilder::random()
        .with_difficulty(0.into())
        .with_parent_hash(parent.id())
        .with_parents_hash(vec![parent.id()])
        .build();

    dag.storage
        .relations_store
        .write()
        .insert(parent.id(), Arc::new(vec![genesis.id()]))?;

    let dag_one = dag.clone();
    let one_clone = one.clone();
    let parent_clone = parent.clone();
    let handle1 = std::thread::spawn(move || {
        dag_one
            .storage
            .relations_store
            .write()
            .insert(one_clone.id(), Arc::new(vec![parent_clone.id()]))
            .expect("failed to insert one");
    });
    let dag_two = dag.clone();
    let two_clone = two.clone();
    let parent_clone = parent.clone();
    let handle2 = std::thread::spawn(move || {
        dag_two
            .storage
            .relations_store
            .write()
            .insert(two_clone.id(), Arc::new(vec![parent_clone.id()]))
            .expect("failed to insert two");
    });

    handle1.join().expect("failed to join handle1");
    handle2.join().expect("failed to join handle2");

    assert!(dag
        .storage
        .relations_store
        .read()
        .get_children(parent.id())?
        .contains(&one.id()));
    assert!(dag
        .storage
        .relations_store
        .read()
        .get_children(parent.id())?
        .contains(&two.id()));

    Ok(())
}

#[test]
fn test_dag_genesis_fork() {
    // initialzie the dag firstly
    let mut dag = BlockDAG::create_for_testing().unwrap();

    let genesis = BlockHeader::dag_genesis_random(0)
        .as_builder()
        .with_difficulty(0.into())
        .build();
    dag.init_with_genesis(genesis.clone()).unwrap();

    // normally add the dag blocks
    let mut parents_hash = vec![genesis.id()];
    for _ in 0..10 {
        let header_builder = BlockHeaderBuilder::random();
        let header = header_builder
            .with_parents_hash(parents_hash.clone())
            .build();
        parents_hash = vec![header.id()];
        dag.commit(header.to_owned(), genesis.parent_hash())
            .unwrap();
        let _ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
    }

    // fork, produce a new dag genesis
    let new_genesis = BlockHeader::dag_genesis_random(0)
        .as_builder()
        .with_difficulty(0.into())
        .build();
    dag.init_with_genesis(new_genesis.clone()).unwrap();

    // record the old dag chain
    let mut old_parents_hash = parents_hash.clone();
    // the new dag chain
    parents_hash = vec![new_genesis.id()];

    // add dag blocks in the old dag chain
    for _ in 0..10 {
        let header_builder = BlockHeaderBuilder::random();
        let header = header_builder
            .with_parents_hash(old_parents_hash.clone())
            .build();
        old_parents_hash = vec![header.id()];
        dag.commit(header.to_owned(), genesis.parent_hash())
            .unwrap();
        let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        println!("add a old header: {:?}, tips: {:?}", header, ghostdata);
    }

    // add dag blocks in the new dag chain
    for _ in 0..10 {
        let header_builder = BlockHeaderBuilder::random();
        let header = header_builder
            .with_parents_hash(parents_hash.clone())
            .build();
        parents_hash = vec![header.id()];
        dag.commit(header.to_owned(), genesis.parent_hash())
            .unwrap();
        let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
    }

    let header_builder = BlockHeaderBuilder::random();
    parents_hash.append(&mut old_parents_hash);
    let header = header_builder.with_parents_hash(parents_hash).build();
    // parents_hash = vec![header.id()];
    dag.commit(header.to_owned(), genesis.parent_hash())
        .unwrap();
    let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
    println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
}

#[test]
fn test_dag_tips_store() {
    let dag = BlockDAG::create_for_testing().unwrap();

    let state = DagState {
        tips: vec![Hash::random()],
    };
    dag.storage
        .state_store
        .write()
        .insert(Hash::zero(), state.clone())
        .expect("failed to store the dag state");

    assert_eq!(
        dag.storage
            .state_store
            .read()
            .get_state_by_hash(Hash::zero())
            .expect("failed to get the dag state"),
        state
    );
}

#[test]
fn test_dag_multiple_commits() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let mut dag = BlockDAG::create_for_testing().unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    // normally add the dag blocks
    let mut parents_hash = vec![genesis.id()];
    let mut parent_hash = genesis.id();
    for i in 2..100 {
        let header_builder = BlockHeaderBuilder::random();
        let header = header_builder
            .with_parent_hash(parent_hash)
            .with_parents_hash(parents_hash.clone())
            .with_number(i)
            .build();
        parents_hash = vec![header.id()];
        parent_hash = header.id();
        dag.commit(header.to_owned(), genesis.parent_hash())?;
        if header.number() == 6 {
            dag.commit(header.to_owned(), genesis.parent_hash())?;
            dag.commit(header.to_owned(), genesis.parent_hash())?;
        }
        let ghostdata = dag.ghostdata(&parents_hash).unwrap();
        println!("add a header: {:?}, tips: {:?}", header, ghostdata);
    }

    Ok(())
}

#[test]
fn test_reachability_abort_add_block() -> anyhow::Result<()> {
    let dag = BlockDAG::create_for_testing().unwrap();
    let reachability_store = dag.storage.reachability_store.clone();

    let mut parent = Hash::random();
    let origin = parent;
    let mut child = Hash::random();
    inquirer::init(reachability_store.write().deref_mut(), parent)?;
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child,
        parent,
        &mut vec![parent].into_iter(),
    )?;

    for i in 0..70 {
        parent = child;
        child = Hash::random();

        inquirer::add_block(
            reachability_store.write().deref_mut(),
            child,
            parent,
            &mut vec![parent].into_iter(),
        )?;
        if (61..=69).contains(&i) {
            for _ in 0..10 {
                inquirer::init(reachability_store.write().deref_mut(), origin)?;
                let result = inquirer::add_block(
                    reachability_store.write().deref_mut(),
                    child,
                    parent,
                    &mut vec![parent].into_iter(),
                );
                match result {
                    Result::Ok(_) => (),
                    Err(ReachabilityError::DataInconsistency) => {
                        let future_covering_set =
                            reachability_store.read().get_future_covering_set(child)?;
                        println!("future_covering_set = {:?}", future_covering_set);
                    }
                    Err(e) => {
                        println!(
                            "failed to add a block in reachability store, error = {:?}",
                            e
                        );
                        bail!("{:?}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_reachability_check_ancestor() -> anyhow::Result<()> {
    let dag = BlockDAG::create_for_testing().unwrap();
    let reachability_store = dag.storage.reachability_store.clone();

    let mut parent = Hash::random();
    let origin = parent;
    let mut child = Hash::random();
    inquirer::init(reachability_store.write().deref_mut(), parent)?;
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child,
        parent,
        &mut vec![].into_iter(),
    )?;
    // mergetset
    let uncle1 = Hash::random();
    let selected_parent_uncle1 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        selected_parent_uncle1,
        parent,
        &mut vec![].into_iter(),
    )?;
    let uncle2 = Hash::random();

    let mut target = child;
    let mut target_parent = parent;
    for i in 0..70 {
        parent = child;
        child = Hash::random();

        if i == 47 {
            inquirer::add_block(
                reachability_store.write().deref_mut(),
                child,
                parent,
                &mut vec![uncle2, uncle1, selected_parent_uncle1].into_iter(),
            )?;

            target = child;
            target_parent = parent;
        } else if i == 46 {
            inquirer::add_block(
                reachability_store.write().deref_mut(),
                child,
                parent,
                &mut vec![].into_iter(),
            )?;
            inquirer::add_block(
                reachability_store.write().deref_mut(),
                uncle1,
                selected_parent_uncle1,
                &mut vec![].into_iter(),
            )?;

            inquirer::add_block(
                reachability_store.write().deref_mut(),
                uncle2,
                parent,
                &mut vec![].into_iter(),
            )?;
        } else {
            inquirer::add_block(
                reachability_store.write().deref_mut(),
                child,
                parent,
                &mut vec![].into_iter(),
            )?;
        }
    }

    // the relationship
    // origin.....target_parent-target.....parent-child
    // ancestor
    assert!(
        dag.check_ancestor_of(selected_parent_uncle1, vec![parent, child])?,
        "failed to check target is the ancestor of its descendant"
    );
    assert!(
        dag.check_ancestor_of(uncle1, vec![parent, child])?,
        "failed to check target is the ancestor of its descendant"
    );
    assert!(
        dag.check_ancestor_of(uncle2, vec![parent, child])?,
        "failed to check target is the ancestor of its descendant"
    );
    assert!(
        dag.check_ancestor_of(target, vec![parent, child])?,
        "failed to check target is the ancestor of its descendant"
    );
    assert!(
        dag.check_ancestor_of(origin, vec![target, parent, child])?,
        "failed to check origin is the parent of its child"
    );
    assert!(
        dag.check_ancestor_of(parent, vec![child])?,
        "failed to check target, parent is the parent of its child"
    );
    assert!(
        dag.check_ancestor_of(target_parent, vec![target])?,
        "failed to check target parent, parent is the parent of its child"
    );

    // not ancestor
    assert!(
        !dag.check_ancestor_of(child, vec![target])?,
        "failed to check child is not the ancestor of its descendant"
    );
    assert!(
        !dag.check_ancestor_of(parent, vec![target])?,
        "failed to check child is not the ancestor of its descendant"
    );
    assert!(
        !dag.check_ancestor_of(child, vec![parent])?,
        "failed to check target, child is the child of its parent"
    );
    assert!(
        !dag.check_ancestor_of(target, vec![target_parent])?,
        "failed to check target is the child of its parent"
    );

    assert!(
        dag.check_ancestor_of(target, vec![Hash::random(), Hash::random(),])
            .is_err(),
        "failed to check not the ancestor of descendants"
    );
    assert!(
        dag.check_ancestor_of(Hash::random(), vec![target, parent, child])
            .is_err(),
        "failed to check not the descendant of parents"
    );

    Ok(())
}

fn print_reachability_data(reachability: &DbReachabilityStore, key: &[Hash]) {
    println!("**********************");
    for k in key {
        let height = reachability.get_height(*k).unwrap();
        let parent = reachability.get_parent(*k).unwrap();
        let children = reachability.get_children(*k).unwrap();
        let interval = reachability.get_interval(*k).unwrap();
        let future_cover_hashes = reachability.get_future_covering_set(*k).unwrap();

        println!("key: {:?}, height: {:?}, interval: {:?}, parent: {:?}, children: {:?}, future_cover_hashes: {:?}", k, height, interval, parent, children, future_cover_hashes);
    }
    println!("**********************");
}

#[test]
fn test_reachability_not_ancestor() -> anyhow::Result<()> {
    let dag = BlockDAG::create_for_testing().unwrap();
    let reachability_store = dag.storage.reachability_store.clone();

    let origin = Hash::random();

    inquirer::init_for_test(
        reachability_store.write().deref_mut(),
        origin,
        Interval::new(1, 1024),
    )?;

    let mut hashes = vec![origin];
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child1 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child1,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child1);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child2 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child2,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child2);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let mut parent = child2;
    for _i in 1..=1000 {
        let child = Hash::random();
        inquirer::add_block(
            reachability_store.write().deref_mut(),
            child,
            parent,
            &mut vec![parent].into_iter(),
        )?;
        hashes.push(child);
        print_reachability_data(reachability_store.read().deref(), &hashes);
        parent = child;
    }

    let child3 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child3,
        parent,
        &mut vec![parent].into_iter(),
    )?;
    hashes.push(child3);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let result = dag.check_ancestor_of(child1, vec![child3]);
    println!("dag.check_ancestor_of() result = {:?}", result);

    Ok(())
}

#[test]
fn test_reachability_algorithm() -> anyhow::Result<()> {
    let dag = BlockDAG::create_for_testing().unwrap();
    let reachability_store = dag.storage.reachability_store.clone();

    let origin = Hash::random();

    inquirer::init_for_test(
        reachability_store.write().deref_mut(),
        origin,
        Interval::new(1, 32),
    )?;

    let mut hashes = vec![origin];
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child1 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child1,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child1);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child2 = Hash::random();
    hashes.push(child2);
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child2,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child3 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child3,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child3);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child4 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child4,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child4);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child5 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child5,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child5);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child6 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child6,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child6);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child7 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child7,
        origin,
        &mut vec![origin].into_iter(),
    )?;
    hashes.push(child7);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    let child8 = Hash::random();
    inquirer::add_block(
        reachability_store.write().deref_mut(),
        child8,
        child1,
        &mut vec![child1].into_iter(),
    )?;
    hashes.push(child8);
    print_reachability_data(reachability_store.read().deref(), &hashes);

    assert!(
        dag.check_ancestor_of(origin, vec![child5])?,
        "child 5 must be origin's child"
    );

    Ok(())
}

fn add_and_print_with_ghostdata(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
    origin: Hash,
    dag: &mut BlockDAG,
    ghostdata: GhostdagData,
) -> anyhow::Result<BlockHeader> {
    let header_builder = BlockHeaderBuilder::random();
    let header = header_builder
        .with_parent_hash(parent)
        .with_parents_hash(parents)
        .with_number(number)
        .build();
    let start = Instant::now();
    dag.commit_trusted_block(header.to_owned(), origin, Arc::new(ghostdata))?;
    let duration = start.elapsed();
    println!(
        "commit header: {:?}, number: {:?}, duration: {:?}",
        header.id(),
        header.number(),
        duration
    );
    let _ghostdata = dag.ghostdata(&[header.id()])?;
    // println!(
    //     "add a header: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
    //     header, ghostdata.mergeset_blues, ghostdata.mergeset_reds, ghostdata.blues_anticone_sizes
    // );
    Ok(header)
}

fn add_and_print_with_difficulty(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
    difficulty: U256,
) -> anyhow::Result<BlockHeader> {
    let header_builder = BlockHeaderBuilder::random();
    let header = header_builder
        .with_parent_hash(parent)
        .with_parents_hash(parents)
        .with_number(number)
        .with_difficulty(difficulty)
        .build();
    let start = Instant::now();
    let duration = start.elapsed();
    println!(
        "commit header: {:?}, number: {:?}, duration: {:?}",
        header.id(),
        header.number(),
        duration
    );

    Ok(header)
}

fn add_and_print_with_pruning_point(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
    origin: Hash,
    pruning_point: Hash,
    dag: &mut BlockDAG,
) -> anyhow::Result<BlockHeader> {
    let header_builder = BlockHeaderBuilder::random();
    let header = header_builder
        .with_parent_hash(parent)
        .with_parents_hash(parents)
        .with_number(number)
        .with_pruning_point(pruning_point)
        .with_difficulty(U256::from(10))
        .build();
    let start = Instant::now();
    dag.commit(header.to_owned(), origin)?;
    let duration = start.elapsed();
    println!(
        "commit header: {:?}, number: {:?}, duration: {:?}",
        header.id(),
        header.number(),
        duration
    );
    // let ghostdata = dag.ghostdata(&[header.id()])?;
    // let ghostdata = dag.ghostdata_by_hash(header.id())?.unwrap();
    // println!(
    //     "add a header: {:?}, selected_parent: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
    //     header, ghostdata.selected_parent, ghostdata.mergeset_blues, ghostdata.mergeset_reds, ghostdata.blues_anticone_sizes
    // );
    Ok(header)
}

fn add_and_print(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
    origin: Hash,
    dag: &mut BlockDAG,
) -> anyhow::Result<BlockHeader> {
    add_and_print_with_pruning_point(number, parent, parents, origin, Hash::zero(), dag)
}

#[test]
fn test_dag_mergeset() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let mut dag = BlockDAG::create_for_testing().unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    println!("add a genesis: {:?}", genesis.id());

    // normally add the dag blocks
    let mut parents_hash = vec![genesis.id()];
    let mut parent_hash = genesis.id();

    let mut header = add_and_print(
        2,
        parent_hash,
        parents_hash,
        genesis.parent_hash(),
        &mut dag,
    )?
    .id();
    let red = add_and_print(3, header, vec![header], genesis.parent_hash(), &mut dag)?.id();

    parents_hash = vec![genesis.id()];
    parent_hash = genesis.id();

    header = add_and_print(
        2,
        parent_hash,
        parents_hash,
        genesis.parent_hash(),
        &mut dag,
    )?
    .id();
    header = add_and_print(3, header, vec![header], genesis.parent_hash(), &mut dag)?.id();
    header = add_and_print(4, header, vec![header], genesis.parent_hash(), &mut dag)?.id();
    let blue = header;

    header = add_and_print(5, blue, vec![blue, red], genesis.parent_hash(), &mut dag)?.id();

    let ghostdata = dag.ghostdata(&[header, red])?;
    println!(
        "add a header: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
        header, ghostdata.mergeset_blues, ghostdata.mergeset_reds, ghostdata.blues_anticone_sizes
    );

    Ok(())
}

#[test]
fn test_big_data_commit() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let mut dag = BlockDAG::create_for_testing().unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    let count = 20000;

    // one
    let mut parent = genesis.clone();
    for i in 0..count {
        let new = add_and_print(
            i + 1,
            parent.id(),
            vec![parent.id()],
            genesis.parent_hash(),
            &mut dag,
        )?;
        parent = new;
    }
    let last_one = parent;

    // two
    let mut parent = genesis.clone();
    for i in 0..count {
        let new = add_and_print(
            i + 1,
            parent.id(),
            vec![parent.id()],
            genesis.parent_hash(),
            &mut dag,
        )?;
        parent = new;
    }
    let last_two = parent;

    let _new = add_and_print(
        count + 1,
        last_one.id(),
        vec![last_one.id(), last_two.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    anyhow::Result::Ok(())
}

#[test]
fn test_prune() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let k = 3;
    let pruning_depth = 4;
    let pruning_finality = 3;

    let mut dag = BlockDAG::create_for_testing_with_parameters(k).unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    let block1 = add_and_print(
        1,
        genesis.id(),
        vec![genesis.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    let block_main_2 = add_and_print(
        2,
        block1.id(),
        vec![block1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_3 = add_and_print(
        3,
        block_main_2.id(),
        vec![block_main_2.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_3_1 = add_and_print(
        3,
        block_main_2.id(),
        vec![block_main_2.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_4 = add_and_print(
        4,
        block_main_3.id(),
        vec![block_main_3.id(), block_main_3_1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_5 = add_and_print(
        5,
        block_main_4.id(),
        vec![block_main_4.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    let block_red_2 = add_and_print(
        2,
        block1.id(),
        vec![block1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_red_2_1 = add_and_print(
        2,
        block1.id(),
        vec![block1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_red_3 = add_and_print(
        3,
        block_red_2.id(),
        vec![block_red_2.id(), block_red_2_1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    // let's obser the blue scores which show how blue the tips are
    let observer1 = dag.ghostdata(&[block_red_3.id()])?;
    println!("observer 1 data: {:?}, ", observer1);

    let observer2 = dag.ghostdata(&[block_red_3.id(), block_main_5.id()])?;
    println!("observer 2 dag data: {:?}, ", observer2);

    let observer3 = dag.ghostdata(&[block_main_5.id()])?;
    println!("observer 3 dag data: {:?}, ", observer3);

    assert!(observer1.blue_score < observer2.blue_score);
    assert!(observer1.selected_parent != observer2.selected_parent);

    assert_eq!(observer3.blue_score, observer2.blue_score);
    assert_eq!(observer3.selected_parent, observer2.selected_parent);

    dag.save_dag_state(
        genesis.id(),
        DagState {
            tips: vec![block_red_3.id(), block_main_5.id()],
        },
    )?;

    // prunning process begins
    let (previous_ghostdata, previous_pruning_point) =
        if block_main_5.pruning_point() == Hash::zero() {
            (
                dag.ghostdata_by_hash(genesis.id())?.ok_or_else(|| {
                    format_err!("failed to get the ghostdata by genesis: {:?}", genesis.id())
                })?,
                genesis.id(),
            )
        } else {
            (
                dag.ghostdata_by_hash(block_main_5.pruning_point())?
                    .ok_or_else(|| {
                        format_err!(
                            "failed to get the ghostdata by pruning point: {:?}",
                            block_main_5.pruning_point()
                        )
                    })?,
                block_main_5.pruning_point(),
            )
        };
    // test the pruning point calculation
    let MineNewDagBlockInfo {
        tips,
        blue_blocks: _,
        pruning_point,
    } = dag.calc_mergeset_and_tips(
        previous_pruning_point,
        previous_ghostdata.as_ref(),
        pruning_depth,
        pruning_finality,
    )?;

    assert_eq!(pruning_point, block_main_2.id());
    assert_eq!(tips.len(), 1);
    assert_eq!(*tips.last().unwrap(), block_main_5.id());

    // test the pruning logic

    let block_main_6 = add_and_print(
        6,
        block_main_5.id(),
        tips.clone(),
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_6_1 =
        add_and_print(6, block_main_5.id(), tips, genesis.parent_hash(), &mut dag)?;
    let block_fork = add_and_print(
        4,
        block_red_3.id(),
        vec![block_red_3.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    dag.save_dag_state(
        genesis.id(),
        DagState {
            tips: vec![block_main_6.id(), block_main_6_1.id(), block_fork.id()],
        },
    )?;

    let MineNewDagBlockInfo {
        tips,
        blue_blocks: _,
        pruning_point,
    } = dag.calc_mergeset_and_tips(
        previous_pruning_point,
        previous_ghostdata.as_ref(),
        pruning_depth,
        pruning_finality,
    )?;

    assert_eq!(pruning_point, block_main_2.id());
    assert_eq!(tips.len(), 2);
    assert_eq!(
        tips.into_iter().collect::<HashSet<_>>(),
        HashSet::from_iter(vec![block_main_6.id(), block_main_6_1.id()])
    );

    anyhow::Result::Ok(())
}

#[test]
fn test_verification_blue_block_inconsistent() -> anyhow::Result<()> {
    loop_to_blue()?;
    anyhow::Result::Ok(())
}

fn loop_to_blue() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let k = 2;

    let mut dag = BlockDAG::create_for_testing_with_parameters(k).unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    let mut storage = HashMap::new();

    let block1 =
        add_and_print_with_difficulty(1, genesis.id(), vec![genesis.id()], U256::from(10))?;
    storage.insert(block1.id(), block1.clone());
    let ghost = dag.ghostdata(&block1.parents())?;
    let verified_ghost = dag.verify_and_ghostdata(
        &ghost
            .mergeset_blues
            .iter()
            .skip(1)
            .cloned()
            .map(|x| storage.get(&x).unwrap().clone())
            .collect::<Vec<_>>(),
        &block1,
    )?;
    dag.commit_trusted_block(
        block1.clone(),
        genesis.parent_hash(),
        Arc::new(verified_ghost),
    )?;

    let mut bottom = vec![];
    let mut last = block1.clone();
    for i in 0..500 {
        let block2 =
            add_and_print_with_difficulty(1 + i, last.id(), vec![last.id()], U256::from(10))?;
        last = block2.clone();
        storage.insert(block2.id(), block2.clone());
        let ghost = dag.ghostdata(&block2.parents())?;
        let verified_ghost = dag.verify_and_ghostdata(
            &ghost
                .mergeset_blues
                .iter()
                .skip(1)
                .cloned()
                .map(|x| storage.get(&x).unwrap().clone())
                .collect::<Vec<_>>(),
            &block2,
        )?;
        dag.commit_trusted_block(
            block2.clone(),
            genesis.parent_hash(),
            Arc::new(verified_ghost),
        )?;
        bottom.push(block2);
    }

    let mut top = vec![];
    let mut iter = bottom.iter().peekable();
    while let Some(first) = iter.next() {
        if let Some(second) = iter.next() {
            let block = add_and_print_with_difficulty(
                3,
                first.id(),
                vec![first.id(), second.id()],
                U256::from(10),
            )?;
            storage.insert(block.id(), block.clone());
            let ghost = dag.ghostdata(&block.parents())?;
            let verified_ghost = dag.verify_and_ghostdata(
                &ghost
                    .mergeset_blues
                    .iter()
                    .skip(1)
                    .cloned()
                    .map(|x| storage.get(&x).unwrap().clone())
                    .collect::<Vec<_>>(),
                &block,
            )?;
            dag.commit_trusted_block(
                block.clone(),
                genesis.parent_hash(),
                Arc::new(verified_ghost),
            )?;

            last = block.clone();
            top.push(block);
        } else {
            let block = add_and_print_with_difficulty(
                3,
                first.id(),
                vec![first.id(), last.id()],
                U256::from(10),
            )?;
            storage.insert(block.id(), block.clone());
            let ghost = dag.ghostdata(&block.parents())?;
            let verified_ghost = dag.verify_and_ghostdata(
                &ghost
                    .mergeset_blues
                    .iter()
                    .skip(1)
                    .cloned()
                    .map(|x| storage.get(&x).unwrap().clone())
                    .collect::<Vec<_>>(),
                &block,
            )?;
            dag.commit_trusted_block(
                block.clone(),
                genesis.parent_hash(),
                Arc::new(verified_ghost),
            )?;

            top.push(block);
            if top.len() == 1 {
                last = top[0].clone();
                break;
            } else {
                bottom.clone_from(&top);
                iter = bottom.iter().peekable();
                top.clear();
            }
        }
    }

    let block1_1 = add_and_print_with_difficulty(
        1,
        genesis.id(),
        vec![last.id(), block1.id()],
        U256::from(99999999),
    )?;
    storage.insert(block1_1.id(), block1_1.clone());
    let ghost = dag.ghostdata(&block1_1.parents())?;
    let verified_ghost = dag.verify_and_ghostdata(
        &ghost
            .mergeset_blues
            .iter()
            .skip(1)
            .cloned()
            .map(|x| storage.get(&x).unwrap().clone())
            .collect::<Vec<_>>(),
        &block1_1,
    )?;
    dag.commit_trusted_block(
        block1_1.clone(),
        genesis.parent_hash(),
        Arc::new(verified_ghost),
    )?;

    let block3 = add_and_print_with_difficulty(
        3,
        block1_1.id(),
        vec![block1_1.id(), last.id()],
        U256::from(10),
    )?;

    let ghostdata = dag.ghostdata(&block3.parents())?;
    println!(
        "add a header: {:?}, selected_parent: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
        block3, ghostdata.selected_parent, ghostdata.mergeset_blues, ghostdata.mergeset_reds, ghostdata.blues_anticone_sizes
    );
    let verified_ghostdata = dag.verify_and_ghostdata(
        &ghostdata
            .mergeset_blues
            .iter()
            .skip(1)
            .map(|x| dag.storage.header_store.get_header(*x).unwrap())
            .collect::<Vec<_>>(),
        &block3,
    )?;
    println!(
        "after verification: selected_parent: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
        verified_ghostdata.selected_parent, verified_ghostdata.mergeset_blues, verified_ghostdata.mergeset_reds, verified_ghostdata.blues_anticone_sizes
    );

    assert_eq!(ghostdata.mergeset_blues, verified_ghostdata.mergeset_blues);

    anyhow::Ok(())
}

#[test]
fn test_verification_blue_block() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let k = 5;

    let mut dag = BlockDAG::create_for_testing_with_parameters(k).unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    let block1 = add_and_print(
        1,
        genesis.id(),
        vec![genesis.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    let block_main_2 = add_and_print(
        2,
        block1.id(),
        vec![block1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_3 = add_and_print(
        3,
        block_main_2.id(),
        vec![block_main_2.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_3_1 = add_and_print(
        3,
        block_main_2.id(),
        vec![block_main_2.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_4 = add_and_print(
        4,
        block_main_3.id(),
        vec![block_main_3.id(), block_main_3_1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_main_5 = add_and_print(
        5,
        block_main_4.id(),
        vec![block_main_4.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    let block_red_2 = add_and_print(
        2,
        block1.id(),
        vec![block1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_red_2_1 = add_and_print(
        2,
        block1.id(),
        vec![block1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_red_3 = add_and_print(
        3,
        block_red_2.id(),
        vec![block_red_2.id(), block_red_2_1.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    // let's obser the blue scores which show how blue the tips are
    let observer1 = dag.ghostdata(&[block_red_3.id()])?;
    println!("observer 1 data: {:?}, ", observer1);

    let observer2 = dag.ghostdata(&[block_red_3.id(), block_main_5.id()])?;
    println!("observer 2 dag data: {:?}, ", observer2);
    assert!(dag
        .ghost_dag_manager()
        .check_ghostdata_blue_block(&observer2)
        .is_ok());

    let mut false_observer2 = observer2.clone();
    let red_block_id = *false_observer2
        .mergeset_reds
        .first()
        .expect("the k is wrong, modify it to create a red block!");
    if red_block_id == block_red_2.id() {
        false_observer2.mergeset_blues = Arc::new(
            vec![red_block_id]
                .into_iter()
                .chain(
                    false_observer2
                        .mergeset_blues
                        .iter()
                        .cloned()
                        .filter(|id| *id != block_red_2_1.id()),
                )
                .collect(),
        );
        false_observer2.mergeset_reds = Arc::new(vec![block_red_2_1.id()]);
    } else {
        false_observer2.mergeset_blues = Arc::new(
            vec![red_block_id]
                .into_iter()
                .chain(
                    false_observer2
                        .mergeset_blues
                        .iter()
                        .cloned()
                        .filter(|id| *id != block_red_2.id()),
                )
                .collect(),
        );
        false_observer2.mergeset_reds = Arc::new(vec![block_red_2.id()]);
    }

    let check_error = dag
        .ghost_dag_manager()
        .check_ghostdata_blue_block(&false_observer2);
    println!(
        "check error: {:?} after the blue block turns red and the red turns blue maliciously",
        check_error
    );
    assert!(check_error.is_err());

    let observer3 = dag.ghostdata(&[block_main_5.id()])?;
    println!("observer 3 dag data: {:?}, ", observer3);

    // assert!(observer1.blue_score < observer2.blue_score);
    // assert!(observer1.selected_parent != observer2.selected_parent);

    // assert_eq!(observer3.blue_score, observer2.blue_score);
    // assert_eq!(observer3.selected_parent, observer2.selected_parent);

    let normal_block = add_and_print(
        6,
        block_main_5.id(),
        vec![block_main_5.id(), block_red_3.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    assert_eq!(
        observer2,
        dag.ghostdata_by_hash(normal_block.id())?
            .expect("the data cannot be none")
            .as_ref()
            .clone()
    );

    let makeup_ghostdata = GhostdagData::new(
        observer2.blue_score,
        observer2.blue_work,
        observer2.selected_parent,
        observer2.mergeset_blues.clone(),
        Arc::new(vec![]),
        HashKTypeMap::new(BlockHashMap::<KType>::new()),
    );
    dag.ghost_dag_manager()
        .check_ghostdata_blue_block(&makeup_ghostdata)?;
    let makeup_block = add_and_print_with_ghostdata(
        6,
        block_main_5.id(),
        vec![block_main_5.id(), block_red_3.id()],
        genesis.parent_hash(),
        &mut dag,
        makeup_ghostdata.clone(),
    )?;

    let block_from_normal = add_and_print(
        7,
        normal_block.id(),
        vec![normal_block.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let block_from_makeup = add_and_print(
        7,
        makeup_block.id(),
        vec![makeup_block.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;

    let ghostdag_data_from_normal = dag
        .ghostdata_by_hash(block_from_normal.id())?
        .expect("the data cannot be none")
        .as_ref()
        .clone();
    let ghostdag_data_from_makeup = dag
        .ghostdata_by_hash(block_from_makeup.id())?
        .expect("the data cannot be none")
        .as_ref()
        .clone();

    println!("normal: {:?}", ghostdag_data_from_normal);
    println!("makeup: {:?}", ghostdag_data_from_makeup);
    assert_eq!(
        ghostdag_data_from_makeup.blue_score,
        ghostdag_data_from_normal.blue_score
    );

    dag.ghost_dag_manager()
        .check_ghostdata_blue_block(&ghostdag_data_from_normal)?;
    dag.ghost_dag_manager()
        .check_ghostdata_blue_block(&ghostdag_data_from_makeup)?;

    let together_mine = dag.ghostdata(&[block_from_normal.id(), block_from_makeup.id()])?;
    let mine_together = add_and_print(
        8,
        together_mine.selected_parent,
        vec![block_from_normal.id(), block_from_makeup.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let together_ghost_data = dag.storage.ghost_dag_store.get_data(mine_together.id())?;
    dag.ghost_dag_manager()
        .check_ghostdata_blue_block(&together_ghost_data)?;

    let together_mine = dag.ghostdata(&[block_from_normal.id(), block_from_makeup.id()])?;
    let mine_together = add_and_print(
        8,
        together_mine.selected_parent,
        vec![block_from_normal.id(), block_from_makeup.id()],
        genesis.parent_hash(),
        &mut dag,
    )?;
    let together_ghost_data = dag.storage.ghost_dag_store.get_data(mine_together.id())?;
    dag.ghost_dag_manager()
        .check_ghostdata_blue_block(&together_ghost_data)?;

    anyhow::Result::Ok(())
}
