

#[cfg(test)]
mod tests {
    use anyhow::{bail, Ok};
    use starcoin_config::RocksdbConfig;
    use starcoin_dag::{blockdag::BlockDAG, consensusdb::{consenses_state::{DagState, DagStateReader, DagStateStore}, prelude::{FlexiDagStorage, FlexiDagStorageConfig}, schemadb::{DbReachabilityStore, ReachabilityStore, ReachabilityStoreReader}}, reachability::{inquirer, reachability_service::ReachabilityService, ReachabilityError}, types::interval::{self, Interval}};
    use starcoin_types::{block::{set_test_flexidag_fork_height, BlockHeader, BlockHeaderBuilder}, blockhash::KType};
    use std::{env, fs, process::ChildStdin};
    use starcoin_crypto::{hash, HashValue as Hash};

    fn build_block_dag(k: KType) -> BlockDAG {
        let db_path = env::temp_dir().join("smolstc");
        if db_path
            .as_path()
            .try_exists()
            .unwrap_or_else(|_| panic!("Failed to check {db_path:?}"))
        {
            fs::remove_dir_all(db_path.as_path()).expect("Failed to delete temporary directory");
        }
        let config = FlexiDagStorageConfig::create_with_params(1, RocksdbConfig::default());
        let db = FlexiDagStorage::create_from_path(db_path, config)
            .expect("Failed to create flexidag storage");
        BlockDAG::new(k, db)
    }

    #[test]
    fn test_dag_0() {
        let mut dag = BlockDAG::create_for_testing().unwrap();
        let genesis = BlockHeader::dag_genesis_random()
            .as_builder()
            .with_difficulty(0.into())
            .build();

        let mut parents_hash = vec![genesis.id()];
        dag.init_with_genesis(genesis.clone()).unwrap();

        for _ in 0..10 {
            let header_builder = BlockHeaderBuilder::random();
            let header = header_builder
                .with_parents_hash(Some(parents_hash.clone()))
                .build();
            parents_hash = vec![header.id()];
            dag.commit(header.to_owned(), genesis.parent_hash()).unwrap();
            let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
            println!("{:?},{:?}", header, ghostdata);
        }
    }

    #[test]
    fn test_dag_1() {
        let genesis = BlockHeader::dag_genesis_random()
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
        let mut dag = build_block_dag(3);
        let expect_selected_parented = vec![block5.id(), block3.id(), block3_1.id(), genesis_id];
        dag.init_with_genesis(genesis.clone()).unwrap();

        dag.commit(block1, genesis.parent_hash()).unwrap();
        dag.commit(block2, genesis.parent_hash()).unwrap();
        dag.commit(block3_1, genesis.parent_hash()).unwrap();
        dag.commit(block3, genesis.parent_hash()).unwrap();
        dag.commit(block4, genesis.parent_hash()).unwrap();
        dag.commit(block5, genesis.parent_hash()).unwrap();
        dag.commit(block6, genesis.parent_hash()).unwrap();
        let mut count = 0;
        while latest_id != genesis_id && count < 4 {
            let ghostdata = dag.ghostdata_by_hash(latest_id).unwrap().unwrap();
            latest_id = ghostdata.selected_parent;
            assert_eq!(expect_selected_parented[count], latest_id);
            count += 1;
        }
    }

    #[tokio::test]
    async fn test_with_spawn() {
        use starcoin_types::block::{BlockHeader, BlockHeaderBuilder};
        let genesis = BlockHeader::dag_genesis_random()
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
        dag.init_with_genesis(genesis.clone()).unwrap();
        dag.commit(block1.clone(), genesis.parent_hash()).unwrap();
        dag.commit(block2.clone(), genesis.parent_hash()).unwrap();
        let block3 = BlockHeaderBuilder::random()
            .with_difficulty(3.into())
            .with_parents_hash(Some(vec![block1.id(), block2.id()]))
            .build();
        let mut handles = vec![];
        for _i in 1..100 {
            let mut dag_clone = dag.clone();
            let block_clone = block3.clone();
            let origin = genesis.parent_hash();
            let handle = tokio::task::spawn_blocking(move || {
                let _ = dag_clone.commit(block_clone, origin);
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
    fn test_dag_genesis_fork() {
        // initialzie the dag firstly
        let mut dag = build_block_dag(3);

        let genesis = BlockHeader::dag_genesis_random()
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
            dag.commit(header.to_owned(), genesis.parent_hash()).unwrap();
            let _ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        }

        // fork, produce a new dag gensis
        let new_genesis = BlockHeader::dag_genesis_random()
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
            dag.commit(header.to_owned(), genesis.parent_hash()).unwrap();
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
            dag.commit(header.to_owned(), genesis.parent_hash()).unwrap();
            let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
            println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
        }

        let header_builder = BlockHeaderBuilder::random();
        parents_hash.append(&mut old_parents_hash);
        let header = header_builder.with_parents_hash(Some(parents_hash)).build();
        // parents_hash = vec![header.id()];
        dag.commit(header.to_owned(), genesis.parent_hash()).unwrap();
        let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
        println!("add a forked header: {:?}, tips: {:?}", header, ghostdata);
    }

    #[test]
    fn test_dag_tips_store() {
        let dag = BlockDAG::create_for_testing().unwrap();

        let state1 = DagState {
            tips: vec![Hash::random()],
        };
        let dag_gensis1 = Hash::random();
        dag.storage.state_store.insert(dag_gensis1, state1.clone()).expect("failed to store the dag state");

        let state2 = DagState {
            tips: vec![Hash::random()],
        };
        let dag_gensis2 = Hash::random();
        dag.storage.state_store.insert(dag_gensis2, state2.clone()).expect("failed to store the dag state");

        assert_eq!(dag.storage.state_store.get_state(dag_gensis1).expect("failed to get the dag state"), state1);
        assert_eq!(dag.storage.state_store.get_state(dag_gensis2).expect("failed to get the dag state"), state2);
    }

    // #[test]
    // fn test_dag_multiple_commits() {
    //     // initialzie the dag firstly
    //     let dag = BlockDAG::create_for_testing().unwrap();

    //     let genesis = BlockHeader::dag_genesis_random()
    //         .as_builder()
    //         .with_difficulty(0.into())
    //         .build();
    //     dag.init_with_genesis(genesis.clone()).unwrap();

    //     // normally add the dag blocks
    //     let mut headers = vec![];
    //     let mut parents_hash = vec![genesis.id()];
    //     let mut parent_hash = genesis.id();
    //     for _ in 0..100 {
    //         let header_builder = BlockHeaderBuilder::random();
    //         let header = header_builder
    //         .with_parent_hash(parent_hash)
    //             .with_parents_hash(Some(parents_hash.clone()))
    //             .build();
    //         parents_hash = vec![header.id()];
    //         parent_hash = header.id();
    //         headers.push(header.clone());
    //         dag.commit(header.to_owned()).unwrap();
    //         let ghostdata = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
    //     }

    //     for _ in 0..10 {
    //         for header in &headers {
    //             let _ = dag.commit(header.clone());
    //             let _ = dag.ghostdata_by_hash(header.id()).unwrap().unwrap();
    //         }
    //     }
    // }

    #[test]
    fn test_dag_multiple_commits() -> anyhow::Result<()> {
        set_test_flexidag_fork_height(1);
        // initialzie the dag firstly
        let mut dag = BlockDAG::create_for_testing().unwrap();

        let origin = BlockHeaderBuilder::random().with_number(0).build();
        let genesis = BlockHeader::dag_genesis_random_with_parent(origin.clone());

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
                println!("commit again: {:?}", header);
                dag.commit(header.to_owned(), genesis.parent_hash())?;
                println!("and again: {:?}", header);
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
      let mut reachability_store = dag.storage.reachability_store.clone();

      let mut parent = Hash::random();
      let origin = parent;
      let mut child = Hash::random();
      inquirer::init(&mut reachability_store, parent)?;
      inquirer::add_block(&mut reachability_store, child, parent, &mut vec![parent].into_iter())?;

      for i in 0..70 {
        parent = child;
        child = Hash::random();

        inquirer::add_block(&mut reachability_store, child, parent, &mut vec![parent].into_iter())?;
        if i >= 61 && i <= 69 {
          for _ in 0..10 {
                inquirer::init(&mut reachability_store, origin)?;
              let result = inquirer::add_block(&mut reachability_store, child, parent, &mut vec![parent].into_iter());
            match result {
                Result::Ok(_) => (),
                Err(ReachabilityError::DataInconsistency) => {
                    let future_covering_set = reachability_store.get_future_covering_set(child)?;
                    println!("future_covering_set = {:?}", future_covering_set);
                    ()
                }
                Err(e) => {
                    println!("failed to add a block in reachability store, error = {:?}", e);
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
      let mut reachability_store = dag.storage.reachability_store.clone();

      let mut parent = Hash::random();
      let origin = parent;
      let mut child = Hash::random();
      inquirer::init(&mut reachability_store, parent)?;
      inquirer::add_block(&mut reachability_store, child, parent, &mut vec![parent].into_iter())?;

      let mut target = child;
      let mut target_parent = parent;
      for i in 0..70 {
        parent = child;
        child = Hash::random();

        if i == 47 {
            inquirer::add_block(&mut reachability_store, child, parent, &mut vec![parent].into_iter())?;

            target = child;
            target_parent = parent;
        } else {
            inquirer::add_block(&mut reachability_store, child, parent, &mut vec![parent].into_iter())?;
        }
      }

      // the relationship
      // origin.....target_parent-target.....parent-child
      // ancestor
      assert!(dag.check_ancestor_of(target, vec![parent, child])?, "failed to check target is the ancestor of its descendant");
      assert!(dag.check_ancestor_of(origin, vec![target, parent, child])?, "failed to check origin is the parent of its child");
      assert!(dag.check_ancestor_of(parent, vec![child])?, "failed to check target, parent is the parent of its child");
      assert!(dag.check_ancestor_of(target_parent, vec![target])?, "failed to check target parent, parent is the parent of its child");

      // not ancestor
      assert!(!dag.check_ancestor_of(child, vec![target])?, "failed to check child is not the ancestor of its descendant");
      assert!(!dag.check_ancestor_of(parent, vec![target])?, "failed to check child is not the ancestor of its descendant");
      assert!(!dag.check_ancestor_of(child, vec![parent])?, "failed to check target, child is the child of its parent");
      assert!(!dag.check_ancestor_of(target, vec![target_parent])?, "failed to check target is the child of its parent");

      assert!(dag.check_ancestor_of(target, vec![Hash::random(), Hash::random(),]).is_err(), "failed to check not the ancestor of descendants");
      assert!(dag.check_ancestor_of(Hash::random(), vec![target, parent, child]).is_err(), "failed to check not the descendant of parents");

      Ok(())
    }

    fn print_reachability_data(reachability: &DbReachabilityStore, key: &[Hash]) {
        println!("**********************");
        for k in key {
            let height = reachability.get_height(*k).unwrap();
            let parent = reachability.get_parent(*k).unwrap();
            let children = reachability.get_children(*k).unwrap();
            let interval = reachability.get_interval(*k).unwrap();

            println!("key: {:?}, height: {:?}, parent: {:?}, children: {:?}, interval: {:?}", k, height, parent, children, interval);
        }
        println!("**********************");
    }

    #[test]
    fn test_reachability_algorighm() -> anyhow::Result<()> {
        let dag = BlockDAG::create_for_testing().unwrap();
        let mut reachability_store = dag.storage.reachability_store.clone();

        let origin = Hash::random();

        inquirer::init_for_test(&mut reachability_store, origin, Interval::new(1, 16))?;

        let mut hashes = vec![origin];
        print_reachability_data(&reachability_store, &hashes);

        let child1 = Hash::random();
        inquirer::add_block(&mut reachability_store, child1, origin, &mut vec![origin].into_iter())?;
        hashes.push(child1);
        print_reachability_data(&reachability_store, &hashes);

        let child2 = Hash::random();
        hashes.push(child2);
        inquirer::add_block(&mut reachability_store, child2, origin, &mut vec![origin].into_iter())?;
        print_reachability_data(&reachability_store, &hashes);

        let child3 = Hash::random();
        inquirer::add_block(&mut reachability_store, child3, origin, &mut vec![origin].into_iter())?;
        hashes.push(child3);
        print_reachability_data(&reachability_store, &hashes);

        let child4 = Hash::random();
        inquirer::add_block(&mut reachability_store, child4, origin, &mut vec![origin].into_iter())?;
        hashes.push(child4);
        print_reachability_data(&reachability_store, &hashes);

        let child5 = Hash::random();
        inquirer::add_block(&mut reachability_store, child5, origin, &mut vec![origin].into_iter())?;
        hashes.push(child5);
        print_reachability_data(&reachability_store, &hashes);

        // let mut count = 6;
        // loop {
        //     let child = Hash::random();
        //     inquirer::add_block(&mut reachability_store, child, origin, &mut vec![origin].into_iter())?;
        //     hashes.push(child);
        //     print!("{count:?}");
        //     print_reachability_data(&reachability_store, &hashes);
        //     count += 1;
        // }

        Ok(())
    }
}
