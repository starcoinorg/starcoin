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
};

use std::{
    collections::HashSet,
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

    for _ in 0..10 {
        let header_builder = BlockHeaderBuilder::random();
        let header = header_builder
            .with_parents_hash(parents_hash.clone())
            .build();
        parents_hash = vec![header.id()];
        dag.commit(header.to_owned())?;
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

    dag.commit(block1)?;
    dag.commit(block2)?;
    dag.commit(block3_1)?;
    dag.commit(block3)?;
    dag.commit(block4)?;
    dag.commit(block5)?;
    dag.commit(block6)?;
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
    dag.commit(block1.clone()).unwrap();
    dag.commit(block2.clone()).unwrap();
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
                match dag_clone.commit(block_clone.clone()) {
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
        dag.commit(header.to_owned()).unwrap();
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
        dag.commit(header.to_owned()).unwrap();
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
        dag.commit(header.to_owned()).unwrap();
        let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
    }

    let header_builder = BlockHeaderBuilder::random();
    parents_hash.append(&mut old_parents_hash);
    let header = header_builder.with_parents_hash(parents_hash).build();
    // parents_hash = vec![header.id()];
    dag.commit(header.to_owned()).unwrap();
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
        dag.commit(header.to_owned())?;
        if header.number() == 6 {
            dag.commit(header.to_owned())?;
            dag.commit(header.to_owned())?;
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
        &mut vec![parent].into_iter(),
    )?;

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
                &mut vec![parent].into_iter(),
            )?;

            target = child;
            target_parent = parent;
        } else {
            inquirer::add_block(
                reachability_store.write().deref_mut(),
                child,
                parent,
                &mut vec![parent].into_iter(),
            )?;
        }
    }

    // the relationship
    // origin.....target_parent-target.....parent-child
    // ancestor
    assert!(
        dag.check_ancestor_of(target, parent)?,
        "failed to check target is the ancestor of its descendant"
    );
    assert!(
        dag.check_ancestor_of(target, child)?,
        "failed to check target is the ancestor of its descendant"
    );
    assert!(
        dag.check_ancestor_of(origin, target)?,
        "failed to check origin is the parent of its child"
    );
    assert!(
        dag.check_ancestor_of(origin, parent)?,
        "failed to check origin is the parent of its child"
    );
    assert!(
        dag.check_ancestor_of(origin, child)?,
        "failed to check origin is the parent of its child"
    );
    assert!(
        dag.check_ancestor_of(parent, child)?,
        "failed to check target, parent is the parent of its child"
    );
    assert!(
        dag.check_ancestor_of(target_parent, target)?,
        "failed to check target parent, parent is the parent of its child"
    );

    // not ancestor
    assert!(
        !dag.check_ancestor_of(child, target)?,
        "failed to check child is not the ancestor of its descendant"
    );
    assert!(
        !dag.check_ancestor_of(parent, target)?,
        "failed to check child is not the ancestor of its descendant"
    );
    assert!(
        !dag.check_ancestor_of(child, parent)?,
        "failed to check target, child is the child of its parent"
    );
    assert!(
        !dag.check_ancestor_of(target, target_parent)?,
        "failed to check target is the child of its parent"
    );

    assert!(
        dag.check_ancestor_of(target, Hash::random()).is_err(),
        "failed to check not the ancestor of descendants"
    );
    assert!(
        dag.check_ancestor_of(Hash::random(), target).is_err(),
        "failed to check not the descendant of parents"
    );
    assert!(
        dag.check_ancestor_of(Hash::random(), parent).is_err(),
        "failed to check not the descendant of parents"
    );
    assert!(
        dag.check_ancestor_of(Hash::random(), child).is_err(),
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

    let result = dag.check_ancestor_of(child1, child3);
    println!("dag.check_ancestor_of() result = {:?}", result);

    Ok(())
}

#[test]
#[ignore = "maxmum data testing for dev"]
fn test_hint_virtaul_selected_parent() -> anyhow::Result<()> {
    let dag = BlockDAG::create_for_testing().unwrap();
    let reachability_store = dag.storage.reachability_store.clone();

    let origin = Hash::random();

    inquirer::init_for_test(
        reachability_store.write().deref_mut(),
        origin,
        // Interval::maximal(),
       Interval::new(1, 100000), 
    )?;

    let mut next_parent = origin;
    let _start = Instant::now();
    for _i in 0..5000 {
        let child = Hash::random();
        inquirer::add_block(
            reachability_store.write().deref_mut(),
            child,
            next_parent,
            &mut vec![].into_iter(),
        )?;
        next_parent = child;
    }
    // println!("add 50000 blocks duration: {:?}", start.elapsed());

    println!("before hint, root reindex = {:?}, origin interval = {:?} ", reachability_store.read().get_reindex_root()?, reachability_store.read().get_interval(origin));
    inquirer::hint_virtual_selected_parent(&mut *reachability_store.write(), next_parent)?;
    println!("after hint, root reindex = {:?}, origin interval = {:?} ", reachability_store.read().get_reindex_root()?, reachability_store.read().get_interval(origin));

    // let start = Instant::now();
    // for _i in 0..200000 {
    //     let child = Hash::random();
    //     inquirer::add_block(
    //         reachability_store.write().deref_mut(),
    //         child,
    //         next_parent,
    //         &mut vec![].into_iter(),
    //     )?;
    //     next_parent = child;
    // }
    // println!("add 50000 blocks again duration: {:?}", start.elapsed());

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
        dag.check_ancestor_of(origin, child5)?,
        "child 5 must be origin's child"
    );

    Ok(())
}

fn add_and_print_with_ghostdata(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
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
    dag.commit_trusted_block(header.to_owned(), Arc::new(ghostdata))?;
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

fn add_and_print_with_pruning_point(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
    pruning_point: Hash,
    dag: &mut BlockDAG,
) -> anyhow::Result<BlockHeader> {
    let header_builder = BlockHeaderBuilder::random();
    let header = header_builder
        .with_parent_hash(parent)
        .with_parents_hash(parents)
        .with_number(number)
        .with_pruning_point(pruning_point)
        .build();
    let start = Instant::now();
    dag.commit(header.to_owned())?;
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

fn add_and_print(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
    dag: &mut BlockDAG,
) -> anyhow::Result<BlockHeader> {
    add_and_print_with_pruning_point(number, parent, parents, Hash::zero(), dag)
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

    let mut header = add_and_print(2, parent_hash, parents_hash, &mut dag)?.id();
    let red = add_and_print(3, header, vec![header], &mut dag)?.id();

    parents_hash = vec![genesis.id()];
    parent_hash = genesis.id();

    header = add_and_print(2, parent_hash, parents_hash, &mut dag)?.id();
    header = add_and_print(3, header, vec![header], &mut dag)?.id();
    header = add_and_print(4, header, vec![header], &mut dag)?.id();
    let blue = header;

    header = add_and_print(5, blue, vec![blue, red], &mut dag)?.id();

    let ghostdata = dag.ghostdata(&[header, red])?;
    println!(
        "add a header: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
        header, ghostdata.mergeset_blues, ghostdata.mergeset_reds, ghostdata.blues_anticone_sizes
    );

    Ok(())
}

#[test]
#[ignore = "this is the large amount of data testing for performance, dev only"]
fn test_big_data_commit() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let mut dag = BlockDAG::create_for_testing().unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    let count = 200000;

    // one
    let mut parent = genesis.clone();
    for i in 0..count {
        let new = add_and_print(i + 1, parent.id(), vec![parent.id()], &mut dag)?;
        if i % 50000 == 0 {
            inquirer::hint_virtual_selected_parent(&mut *dag.storage.reachability_store.write(), new.id())?;
        }
        parent = new;
    }
    // let last_one = parent;

    // // two
    // let mut parent = genesis.clone();
    // for i in 0..count {
    //     let new = add_and_print(
    //         i + 1,
    //         parent.id(),
    //         vec![parent.id()],
    //         genesis.parent_hash(),
    //         &mut dag,
    //     )?;
    //     parent = new;
    // }
    // let last_two = parent;

    // let _new = add_and_print(
    //     count + 1,
    //     last_one.id(),
    //     vec![last_one.id(), last_two.id()],
    //     genesis.parent_hash(),
    //     &mut dag,
    // )?;

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

    let block1 = add_and_print(1, genesis.id(), vec![genesis.id()], &mut dag)?;

    let block_main_2 = add_and_print(2, block1.id(), vec![block1.id()], &mut dag)?;
    let block_main_3 = add_and_print(3, block_main_2.id(), vec![block_main_2.id()], &mut dag)?;
    let block_main_3_1 = add_and_print(3, block_main_2.id(), vec![block_main_2.id()], &mut dag)?;
    let block_main_4 = add_and_print(
        4,
        block_main_3.id(),
        vec![block_main_3.id(), block_main_3_1.id()],
        &mut dag,
    )?;
    let block_main_5 = add_and_print(5, block_main_4.id(), vec![block_main_4.id()], &mut dag)?;

    let block_red_2 = add_and_print(2, block1.id(), vec![block1.id()], &mut dag)?;
    let block_red_2_1 = add_and_print(2, block1.id(), vec![block1.id()], &mut dag)?;
    let block_red_3 = add_and_print(
        3,
        block_red_2.id(),
        vec![block_red_2.id(), block_red_2_1.id()],
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

    let block_main_6 = add_and_print(6, block_main_5.id(), tips.clone(), &mut dag)?;
    let block_main_6_1 = add_and_print(6, block_main_5.id(), tips, &mut dag)?;
    let block_fork = add_and_print(4, block_red_3.id(), vec![block_red_3.id()], &mut dag)?;

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
fn test_verification_blue_block() -> anyhow::Result<()> {
    // initialzie the dag firstly
    let k = 5;

    let mut dag = BlockDAG::create_for_testing_with_parameters(k).unwrap();

    let origin = BlockHeaderBuilder::random().with_number(0).build();
    let genesis = BlockHeader::dag_genesis_random_with_parent(origin)?;

    dag.init_with_genesis(genesis.clone()).unwrap();

    let block1 = add_and_print(1, genesis.id(), vec![genesis.id()], &mut dag)?;

    let block_main_2 = add_and_print(2, block1.id(), vec![block1.id()], &mut dag)?;
    let block_main_3 = add_and_print(3, block_main_2.id(), vec![block_main_2.id()], &mut dag)?;
    let block_main_3_1 = add_and_print(3, block_main_2.id(), vec![block_main_2.id()], &mut dag)?;
    let block_main_4 = add_and_print(
        4,
        block_main_3.id(),
        vec![block_main_3.id(), block_main_3_1.id()],
        &mut dag,
    )?;
    let block_main_5 = add_and_print(5, block_main_4.id(), vec![block_main_4.id()], &mut dag)?;

    let block_red_2 = add_and_print(2, block1.id(), vec![block1.id()], &mut dag)?;
    let block_red_2_1 = add_and_print(2, block1.id(), vec![block1.id()], &mut dag)?;
    let block_red_3 = add_and_print(
        3,
        block_red_2.id(),
        vec![block_red_2.id(), block_red_2_1.id()],
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
        &mut dag,
        makeup_ghostdata.clone(),
    )?;

    let block_from_normal = add_and_print(7, normal_block.id(), vec![normal_block.id()], &mut dag)?;
    let block_from_makeup = add_and_print(7, makeup_block.id(), vec![makeup_block.id()], &mut dag)?;

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
        &mut dag,
    )?;
    let together_ghost_data = dag.storage.ghost_dag_store.get_data(mine_together.id())?;
    dag.ghost_dag_manager()
        .check_ghostdata_blue_block(&together_ghost_data)?;

    anyhow::Result::Ok(())
}
