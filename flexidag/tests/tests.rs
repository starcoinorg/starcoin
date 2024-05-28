// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Ok, Result};
use starcoin_crypto::HashValue as Hash;
use starcoin_dag::{
    blockdag::BlockDAG,
    consensusdb::{
        consenses_state::{DagState, DagStateReader, DagStateStore},
        schemadb::{
            DbReachabilityStore, ReachabilityStore, ReachabilityStoreReader, RelationsStore,
            RelationsStoreReader,
        },
    },
    reachability::{inquirer, ReachabilityError},
    types::interval::Interval,
};
use starcoin_logger::prelude::debug;
use starcoin_types::block::{BlockHeader, BlockHeaderBuilder, BlockNumber};

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
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
            .with_parents_hash(Some(parents_hash.clone()))
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
        .with_parents_hash(Some(vec![genesis.id()]))
        .build();
    let block2 = BlockHeaderBuilder::random()
        .with_difficulty(2.into())
        .with_parents_hash(Some(vec![genesis.id()]))
        .build();
    let block3_1 = BlockHeaderBuilder::random()
        .with_difficulty(1.into())
        .with_parents_hash(Some(vec![genesis.id()]))
        .build();
    let block3 = BlockHeaderBuilder::random()
        .with_difficulty(3.into())
        .with_parents_hash(Some(vec![block3_1.id()]))
        .build();
    let block4 = BlockHeaderBuilder::random()
        .with_difficulty(4.into())
        .with_parents_hash(Some(vec![block1.id(), block2.id()]))
        .build();
    let block5 = BlockHeaderBuilder::random()
        .with_difficulty(4.into())
        .with_parents_hash(Some(vec![block2.id(), block3.id()]))
        .build();
    let block6 = BlockHeaderBuilder::random()
        .with_difficulty(5.into())
        .with_parents_hash(Some(vec![block4.id(), block5.id()]))
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
        .with_parents_hash(Some(vec![genesis.id()]))
        .build();
    let block2 = BlockHeaderBuilder::random()
        .with_difficulty(2.into())
        .with_parents_hash(Some(vec![genesis.id()]))
        .build();
    let mut dag = BlockDAG::create_for_testing().unwrap();
    let real_origin = dag.init_with_genesis(genesis.clone()).unwrap();
    dag.commit(block1.clone(), real_origin).unwrap();
    dag.commit(block2.clone(), real_origin).unwrap();
    let block3 = BlockHeaderBuilder::random()
        .with_difficulty(3.into())
        .with_parents_hash(Some(vec![block1.id(), block2.id()]))
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
        .with_parents_hash(Some(vec![parent.id()]))
        .build();

    let two = BlockHeaderBuilder::random()
        .with_difficulty(0.into())
        .with_parent_hash(parent.id())
        .with_parents_hash(Some(vec![parent.id()]))
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
            .with_parents_hash(Some(parents_hash.clone()))
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
            .with_parents_hash(Some(old_parents_hash.clone()))
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
            .with_parents_hash(Some(parents_hash.clone()))
            .build();
        parents_hash = vec![header.id()];
        dag.commit(header.to_owned(), genesis.parent_hash())
            .unwrap();
        let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
    }

    let header_builder = BlockHeaderBuilder::random();
    parents_hash.append(&mut old_parents_hash);
    let header = header_builder.with_parents_hash(Some(parents_hash)).build();
    // parents_hash = vec![header.id()];
    dag.commit(header.to_owned(), genesis.parent_hash())
        .unwrap();
    let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
    println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
}

#[test]
fn test_dag_tips_store() {
    let dag = BlockDAG::create_for_testing().unwrap();

    let state1 = DagState {
        tips: vec![Hash::random()],
    };
    let dag_genesis1 = Hash::random();
    dag.storage
        .state_store
        .write()
        .insert(dag_genesis1, state1.clone())
        .expect("failed to store the dag state");

    let state2 = DagState {
        tips: vec![Hash::random()],
    };
    let dag_genesis2 = Hash::random();
    dag.storage
        .state_store
        .write()
        .insert(dag_genesis2, state2.clone())
        .expect("failed to store the dag state");

    assert_eq!(
        dag.storage
            .state_store
            .read()
            .get_state(dag_genesis1)
            .expect("failed to get the dag state"),
        state1
    );
    assert_eq!(
        dag.storage
            .state_store
            .read()
            .get_state(dag_genesis2)
            .expect("failed to get the dag state"),
        state2
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
            .with_parents_hash(Some(parents_hash.clone()))
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
fn test_reachability_algorighm() -> anyhow::Result<()> {
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

fn add_and_print(
    number: BlockNumber,
    parent: Hash,
    parents: Vec<Hash>,
    origin: Hash,
    dag: &mut BlockDAG,
) -> anyhow::Result<Hash> {
    let header_builder = BlockHeaderBuilder::random();
    let header = header_builder
        .with_parent_hash(parent)
        .with_parents_hash(Some(parents))
        .with_number(number)
        .build();
    dag.commit(header.to_owned(), origin)?;
    let ghostdata = dag.ghostdata(&[header.id()])?;
    println!(
        "add a header: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
        header, ghostdata.mergeset_blues, ghostdata.mergeset_reds, ghostdata.blues_anticone_sizes
    );
    Ok(header.id())
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
    )?;
    let red = add_and_print(3, header, vec![header], genesis.parent_hash(), &mut dag)?;

    parents_hash = vec![genesis.id()];
    parent_hash = genesis.id();

    header = add_and_print(
        2,
        parent_hash,
        parents_hash,
        genesis.parent_hash(),
        &mut dag,
    )?;
    header = add_and_print(3, header, vec![header], genesis.parent_hash(), &mut dag)?;
    header = add_and_print(4, header, vec![header], genesis.parent_hash(), &mut dag)?;
    let blue = header;

    header = add_and_print(5, blue, vec![blue, red], genesis.parent_hash(), &mut dag)?;

    let ghostdata = dag.ghostdata(&[header, red])?;
    println!(
        "add a header: {:?}, blue set: {:?}, red set: {:?}, blue anticone size: {:?}",
        header, ghostdata.mergeset_blues, ghostdata.mergeset_reds, ghostdata.blues_anticone_sizes
    );

    Ok(())
}
