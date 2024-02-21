// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verifier::{BlockVerifier, FullVerifier};
use anyhow::{bail, ensure, format_err, Result};
use once_cell::sync::Lazy;
use sp_utils::stop_watch::{watch, CHAIN_WATCH_NAME};
use starcoin_accumulator::inmemory::InMemoryAccumulator;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_chain_api::{
    verify_block, ChainReader, ChainWriter, ConnectBlockError, EventWithProof, ExcludedTxns,
    ExecutedBlock, MintedUncleNumber, TransactionInfoWithProof, VerifiedBlock, VerifyBlockField,
};
use starcoin_consensus::Consensus;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_executor::{BlockExecutedData, VMMetrics};
use starcoin_logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::{AccountStateReader, ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Store;
use starcoin_time_service::TimeService;
use starcoin_types::block::BlockIdAndNumber;
use starcoin_types::contract_event::ContractEventInfo;
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::RichTransactionInfo;
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    contract_event::ContractEvent,
    error::BlockExecutorError,
    transaction::{SignedUserTransaction, Transaction},
    U256,
};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::on_chain_resource::Epoch;
use std::cmp::min;
use std::collections::BTreeMap;
use std::iter::Extend;
use std::option::Option::{None, Some};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{collections::HashMap, sync::Arc};

static MAIN_DIRECT_SAVE_BLOCK_HASH_MAP: Lazy<BTreeMap<HashValue, (BlockExecutedData, BlockInfo)>> =
    Lazy::new(|| {
        let mut maps = BTreeMap::new();

        // 16450410
        maps.insert(
        HashValue::from_hex_literal(
            "0x6f36ea7df4bedb8e8aefebd822d493fb95c9434ae1d5095c0f5f2d7c33e7b866",
        )
        .unwrap(),
        (
            serde_json::from_str("{\"state_root\":\"0xdc2e677859ac1a318a8f3f76bf6ace0573712a328ac626511e2f1cc086603db4\",\"txn_infos\":[{\"transaction_hash\":\"0x0add4124674a011152aeda4f08fa92949a20595e7a97e386f73f597f106acecb\",\"state_root_hash\":\"0x2262169c08c8c0103b87e56d64ae069f66455037a54ff12c8d753f55942c1472\",\"event_root_hash\":\"0xe970c5bbb07a92ad979ef323bd52d89878e9b7d0be00f93a1249e5b00f6fbcd3\",\"gas_used\":0,\"status\":\"Executed\"},{\"transaction_hash\":\"0xf633ba6626472f9cdf149e7fbfa835f1bb9d4b95990c456107606436df379cb5\",\"state_root_hash\":\"0xdc2e677859ac1a318a8f3f76bf6ace0573712a328ac626511e2f1cc086603db4\",\"event_root_hash\":\"0x414343554d554c41544f525f504c414345484f4c4445525f4841534800000000\",\"gas_used\":7800,\"status\":\"Executed\"}],\"txn_events\":[[{\"V0\":{\"key\":\"0x040000000000000000000000000000000000000000000001\",\"sequence_number\":16450409,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"Block\",\"name\":\"NewBlockEvent\",\"type_args\":[]}},\"event_data\":[106,3,251,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,228,188,23,80,141,1,0,0,0,0,0,0,0,0,0,0]}},{\"V0\":{\"key\":\"0x100000000000000000000000000000000000000000000001\",\"sequence_number\":16450405,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"Treasury\",\"name\":\"WithdrawEvent\",\"type_args\":[]}},\"event_data\":[0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0]}},{\"V0\":{\"key\":\"0x0100000000000000707d8fc016acae0a1a859769ad0c4fcf\",\"sequence_number\":9254139,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"Account\",\"name\":\"DepositEvent\",\"type_args\":[]}},\"event_data\":[0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,83,84,67,3,83,84,67,0]}},{\"V0\":{\"key\":\"0x0d0000000000000000000000000000000000000000000001\",\"sequence_number\":16450402,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"BlockReward\",\"name\":\"BlockRewardEvent\",\"type_args\":[]}},\"event_data\":[99,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207]}}],[]],\"txn_table_infos\":{},\"write_sets\":[{\"write_set\":[[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Block::BlockMetadata\"},{\"Value\":[106,3,251,0,0,0,0,0,32,167,250,142,100,104,137,167,195,19,235,251,216,52,56,126,189,43,174,112,140,77,48,173,131,173,174,216,163,176,77,197,160,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,106,3,251,0,0,0,0,0,24,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::BlockReward::RewardQueue\"},{\"Value\":[99,3,251,0,0,0,0,0,7,100,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,217,178,213,110,141,32,169,17,178,220,89,41,105,95,78,192,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,101,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,102,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,103,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,104,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,105,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,106,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,99,3,251,0,0,0,0,0,24,13,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Epoch::EpochData\"},{\"Value\":[2,0,0,0,0,0,0,0,0,208,183,43,106,0,0,0,0,0,0,0,0,0,0,0,242,10,5,0,0,0,0,0,0,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Timestamp::CurrentTimeMilliseconds\"},{\"Value\":[228,188,23,80,141,1,0,0]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Treasury::Treasury<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[64,59,201,110,88,254,155,39,0,0,0,0,0,0,0,0,102,3,251,0,0,0,0,0,24,16,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,24,17,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]}],[{\"AccessPath\":\"0x707d8fc016acae0a1a859769ad0c4fcf/1/0x00000000000000000000000000000001::Account::Account\"},{\"Value\":[32,219,102,253,225,28,213,194,6,87,21,101,142,191,225,171,163,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,1,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,1,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,30,0,0,0,0,0,0,0,24,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,252,52,141,0,0,0,0,0,24,1,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,1,0,0,0,0,0,0,0,24,2,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,30,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x707d8fc016acae0a1a859769ad0c4fcf/1/0x00000000000000000000000000000001::Account::Balance<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[117,203,24,103,224,197,50,0,0,0,0,0,0,0,0,0]}]]},{\"write_set\":[[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::TransactionFee::TransactionFee<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[120,30,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x12d95e1db2a54d15bc50927e5655af2d/1/0x00000000000000000000000000000001::Account::Account\"},{\"Value\":[32,147,37,155,150,127,92,178,4,153,120,90,251,58,181,3,135,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,0,0,0,0,0,0,0,0,24,0,0,0,0,0,0,0,0,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,0,0,0,0,0,0,0,24,1,0,0,0,0,0,0,0,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,0,0,0,0,0,0,0,24,2,0,0,0,0,0,0,0,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x12d95e1db2a54d15bc50927e5655af2d/1/0x00000000000000000000000000000001::Account::Balance<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[120,87,231,236,96,0,0,0,0,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x12d95e1db2a54d15bc50927e5655af2d/0/LocalPool\"},{\"Value\":[161,28,235,11,6,0,0,0,8,1,0,6,2,6,6,3,12,12,4,24,2,5,26,11,7,37,32,8,69,32,12,101,13,0,0,1,1,1,2,2,2,4,1,0,1,0,3,0,1,1,4,1,3,0,1,1,4,1,2,2,5,11,0,1,9,0,0,1,9,0,9,76,111,99,97,108,80,111,111,108,7,65,99,99,111,117,110,116,5,84,111,107,101,110,7,100,101,112,111,115,105,116,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,4,0,1,4,11,0,11,1,56,0,2,0]}]]}]}"
        ).unwrap(),
            serde_json::from_str("{\"block_id\":\"0x6f36ea7df4bedb8e8aefebd822d493fb95c9434ae1d5095c0f5f2d7c33e7b866\",\"total_difficulty\":\"0x0e5ccf62ca60af\",\"txn_accumulator_info\":{\"accumulator_root\":\"0x527954e38cd42f439116cd3d68d7028050d08af9cdac35f1914f4bd03e62c975\",\"frozen_subtree_roots\":[\"0x6a7660543beade1a82985e4d40914c5e375a3b1b1ef8c59b7fb0d600e94e6d0d\",\"0xe875ab19dbe0363c9d7e7f92ee5b42398b057893db298734ef3d90033ead0ced\",\"0x9d031047c542091ad7e64b34145b8e7d41b835a755f49ef34b2da18e7589a4a5\",\"0x3aaace42e991c7cff9ebdd0140b89f8332475161aba28b8870d8904fa249a43b\",\"0xc6137ee8852c1813ad171cda4f80f0aa3d95ceddbe2f481caf244bf131c41251\",\"0x9cd52e03d9f9b6e83922c4bbd1ae4875d70d19b9661e766d224605032a4c5877\",\"0xf4503b61dee640fbbca99585bdbdcdc5231a5cdb443a6626eeb5c28502536c23\",\"0x199a4b7b0ac05cae4cfc1d9205ecd97a2e21c6d6fe93620ee2180dd8192ff7bb\",\"0x96bde2f8ff1348229cd5e9322fb3c0c2f27cbe0c107fc7610576090a8ca7acc7\",\"0xc83dd4240fee65b4fcb72d5c3f746ba36d401fd3dd2015aa60a1579a7d5592d6\"],\"num_leaves\":17974228,\"num_nodes\":35948446},\"block_accumulator_info\":{\"accumulator_root\":\"0xfa69a779d77202b0884505cb002eebc94c8f4ed49064033b875fdaa4322356a4\",\"frozen_subtree_roots\":[\"0x0932a6e4be6739c479d01adf8d5f82576f5befe096cc6a2ac866388b6ec7ae9c\",\"0x21e59980403d2675dbe227bec852f6cf781aeff625d9d8d9cb9f609100a60e0d\",\"0xe8b3a3d48b5882e3a9332c04433a7dc72477ec0cb4428ffeb54bc17369a569af\",\"0x723d6047a914757868553ab53927d882ac97911224bcce941034101e10491ee1\",\"0xb8635f1f523745fdda04448f737e57e3a3c42a29e01b5bbab849fa67468be24d\",\"0x8c70adf9632d404b7b3a90960894795cc68f47f112735683b9c8bd78a1470d1e\",\"0xcf03f3704e1b19c006ee86d34334fe55064da226bdb25bb99ba3b2821bac079e\",\"0x318314295715b2869b9d1f385233801cf87ca840225889216f71c3e8f8afb336\",\"0x805f28777f6244389247f4bc1cdc5fe35f0a0e356f3f9391447e3b7d7127ef7d\",\"0x28ed45eda11bac5a6e875bf7a0b8a4473d8f176acf406c5badb06ef353b832de\",\"0xc6ec01cd217b2f3080bd123cf2d0d58f9db080eac4e26f599803c1b271319e2a\",\"0x7b8619c0799d82f97e74c2184cb64f0491fc1fb70add12a5148ec70fdc94d351\",\"0xbe6b2401f4c7bd022c8fb32b52e37da37ab219d5966735e275a0e137d2c6bde7\"],\"num_leaves\":16450410,\"num_nodes\":32900807}}").unwrap()
        )
    );
        //16450487
        maps.insert(
            HashValue::from_hex_literal(
                "0x6ece280add39a309690c177a36f401eecefa79c69e1ec02dd2cd6b3b33e1eb62",
            )
                .unwrap(),
            (
         serde_json::from_str("{\"state_root\":\"0x791fddf28ff6e42c934c3666e539e749c313e01bc3ba7c3c2fc99be6979c35d5\",\"txn_infos\":[{\"transaction_hash\":\"0xd4fe82d70539e762e0122190df1f18c220c0ad3c380afb531eea4f3480e39c76\",\"state_root_hash\":\"0x3a9cef1dbbc1051b82979232ffd35ac18bd0b5e577ebd4c5f756f41281e85964\",\"event_root_hash\":\"0x567c0b0d3389952178ebcd5ed8568b49b73e5f99cc1683fc77564bf2df28371b\",\"gas_used\":0,\"status\":\"Executed\"},{\"transaction_hash\":\"0xbac88e683630548e3ef5630ac417a149a2c843f18825aaf361f93587b842715e\",\"state_root_hash\":\"0x791fddf28ff6e42c934c3666e539e749c313e01bc3ba7c3c2fc99be6979c35d5\",\"event_root_hash\":\"0x414343554d554c41544f525f504c414345484f4c4445525f4841534800000000\",\"gas_used\":25611,\"status\":{\"MoveAbort\":[{\"Module\":{\"address\":\"0x00000000000000000000000000000001\",\"name\":\"Account\"}},27141]}}],\"txn_events\":[[{\"V0\":{\"key\":\"0x040000000000000000000000000000000000000000000001\",\"sequence_number\":16450486,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"Block\",\"name\":\"NewBlockEvent\",\"type_args\":[]}},\"event_data\":[183,3,251,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,148,237,29,80,141,1,0,0,0,0,0,0,0,0,0,0]}},{\"V0\":{\"key\":\"0x100000000000000000000000000000000000000000000001\",\"sequence_number\":16450482,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"Treasury\",\"name\":\"WithdrawEvent\",\"type_args\":[]}},\"event_data\":[0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0]}},{\"V0\":{\"key\":\"0x0100000000000000707d8fc016acae0a1a859769ad0c4fcf\",\"sequence_number\":9254206,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"Account\",\"name\":\"DepositEvent\",\"type_args\":[]}},\"event_data\":[0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,83,84,67,3,83,84,67,0]}},{\"V0\":{\"key\":\"0x0d0000000000000000000000000000000000000000000001\",\"sequence_number\":16450479,\"type_tag\":{\"struct\":{\"address\":\"0x00000000000000000000000000000001\",\"module\":\"BlockReward\",\"name\":\"BlockRewardEvent\",\"type_args\":[]}},\"event_data\":[176,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207]}}],[]],\"txn_table_infos\":{},\"write_sets\":[{\"write_set\":[[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Block::BlockMetadata\"},{\"Value\":[183,3,251,0,0,0,0,0,32,21,110,11,138,83,157,97,209,46,164,133,38,1,110,27,167,44,224,41,53,137,110,200,250,171,55,171,72,107,255,68,73,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,183,3,251,0,0,0,0,0,24,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::BlockReward::RewardQueue\"},{\"Value\":[176,3,251,0,0,0,0,0,7,177,3,251,0,0,0,0,0,0,87,211,71,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,178,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,179,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,180,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,181,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,182,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,183,3,251,0,0,0,0,0,0,242,5,42,1,0,0,0,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,176,3,251,0,0,0,0,0,24,13,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Epoch::EpochData\"},{\"Value\":[6,0,0,0,0,0,0,0,0,46,183,70,196,0,0,0,0,0,0,0,0,0,0,0,163,206,5,0,0,0,0,0,0,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Timestamp::CurrentTimeMilliseconds\"},{\"Value\":[148,237,29,80,141,1,0,0]}],[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::Treasury::Treasury<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[64,66,151,113,254,253,155,39,0,0,0,0,0,0,0,0,179,3,251,0,0,0,0,0,24,16,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,24,17,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]}],[{\"AccessPath\":\"0x707d8fc016acae0a1a859769ad0c4fcf/1/0x00000000000000000000000000000001::Account::Account\"},{\"Value\":[32,219,102,253,225,28,213,194,6,87,21,101,142,191,225,171,163,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,1,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,1,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,30,0,0,0,0,0,0,0,24,0,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,63,53,141,0,0,0,0,0,24,1,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,1,0,0,0,0,0,0,0,24,2,0,0,0,0,0,0,0,112,125,143,192,22,172,174,10,26,133,151,105,173,12,79,207,30,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x707d8fc016acae0a1a859769ad0c4fcf/1/0x00000000000000000000000000000001::Account::Balance<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[38,20,16,192,46,198,50,0,0,0,0,0,0,0,0,0]}]]},{\"write_set\":[[{\"AccessPath\":\"0x00000000000000000000000000000001/1/0x00000000000000000000000000000001::TransactionFee::TransactionFee<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[11,100,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x12d95e1db2a54d15bc50927e5655af2d/1/0x00000000000000000000000000000001::Account::Account\"},{\"Value\":[32,147,37,155,150,127,92,178,4,153,120,90,251,58,181,3,135,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,0,0,0,0,0,0,0,0,24,0,0,0,0,0,0,0,0,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,0,0,0,0,0,0,0,24,1,0,0,0,0,0,0,0,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,1,0,0,0,0,0,0,0,24,2,0,0,0,0,0,0,0,18,217,94,29,178,165,77,21,188,80,146,126,86,85,175,45,2,0,0,0,0,0,0,0]}],[{\"AccessPath\":\"0x12d95e1db2a54d15bc50927e5655af2d/1/0x00000000000000000000000000000001::Account::Balance<0x00000000000000000000000000000001::STC::STC>\"},{\"Value\":[109,243,230,236,96,0,0,0,0,0,0,0,0,0,0,0]}]]}]}" ).unwrap(),
         serde_json::from_str("{\"block_id\":\"0x6ece280add39a309690c177a36f401eecefa79c69e1ec02dd2cd6b3b33e1eb62\",\"total_difficulty\":\"0x0e5cd2c49a326e\",\"txn_accumulator_info\":{\"accumulator_root\":\"0x9e0886abfc7f98a598f08811236cae4a9dcb186f3bd1f890bfb2f29813a18e05\",\"frozen_subtree_roots\":[\"0x6a7660543beade1a82985e4d40914c5e375a3b1b1ef8c59b7fb0d600e94e6d0d\",\"0xe875ab19dbe0363c9d7e7f92ee5b42398b057893db298734ef3d90033ead0ced\",\"0x9d031047c542091ad7e64b34145b8e7d41b835a755f49ef34b2da18e7589a4a5\",\"0x3aaace42e991c7cff9ebdd0140b89f8332475161aba28b8870d8904fa249a43b\",\"0x509be32124bf78eb4ddfc24a57737bde2ff4e7cd45a4c6108227daded95ec65c\",\"0x8eceebc1be7879992ff2493874e4c6d191109bd2e505b6e7c093e2eabbeff7cf\",\"0x33da89893b32d0570f72012a5e5091a15f2e760c29cbaf020c9dcec5acbfae78\",\"0x413f606297d06bda84367d6db0f038aac167dd56c896192252c625de7bcdba24\"],\"num_leaves\":17974307,\"num_nodes\":35948606},\"block_accumulator_info\":{\"accumulator_root\":\"0xc1eb7302156f3557e6e1c51975d1e9aa7ca4b30d2c9ec89b22f630f53bfa2903\",\"frozen_subtree_roots\":[\"0x0932a6e4be6739c479d01adf8d5f82576f5befe096cc6a2ac866388b6ec7ae9c\",\"0x21e59980403d2675dbe227bec852f6cf781aeff625d9d8d9cb9f609100a60e0d\",\"0xe8b3a3d48b5882e3a9332c04433a7dc72477ec0cb4428ffeb54bc17369a569af\",\"0x723d6047a914757868553ab53927d882ac97911224bcce941034101e10491ee1\",\"0xb8635f1f523745fdda04448f737e57e3a3c42a29e01b5bbab849fa67468be24d\",\"0x8c70adf9632d404b7b3a90960894795cc68f47f112735683b9c8bd78a1470d1e\",\"0xcf03f3704e1b19c006ee86d34334fe55064da226bdb25bb99ba3b2821bac079e\",\"0x318314295715b2869b9d1f385233801cf87ca840225889216f71c3e8f8afb336\",\"0x805f28777f6244389247f4bc1cdc5fe35f0a0e356f3f9391447e3b7d7127ef7d\",\"0x604e3ad65b8a66af542d207a19a048a389b7732d524041e40cbebee510d581fa\",\"0x73c28b55dd18d2adce28040ba77f67b6981b1f481f9dbfc4a3f7400ba8b90ce2\",\"0x5d7a287f03f514afcbc7560f33c6b3d2e383882871f68e75edc01873abc49b0c\",\"0x85f278469e45368c5c1ae0b642756c99cb56781289694d9c9ce0ec973b8f585a\",\"0xe546f4577d81a7481ef66f179c0501bcbe2b0056da50e7a78b05eed70c254ee3\",\"0x156e0b8a539d61d12ea48526016e1ba72ce02935896ec8faab37ab486bff4449\"],\"num_leaves\":16450487,\"num_nodes\":32900959}}").unwrap()
            )
                );
        maps
    });

static OUTPUT_BLOCK: AtomicBool = AtomicBool::new(false);

pub struct ChainStatusWithBlock {
    pub status: ChainStatus,
    pub head: Block,
}

pub struct BlockChain {
    genesis_hash: HashValue,
    txn_accumulator: MerkleAccumulator,
    block_accumulator: MerkleAccumulator,
    status: ChainStatusWithBlock,
    statedb: ChainStateDB,
    storage: Arc<dyn Store>,
    time_service: Arc<dyn TimeService>,
    uncles: HashMap<HashValue, MintedUncleNumber>,
    epoch: Epoch,
    vm_metrics: Option<VMMetrics>,
}

impl BlockChain {
    pub fn new(
        time_service: Arc<dyn TimeService>,
        head_block_hash: HashValue,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let head = storage
            .get_block_by_hash(head_block_hash)?
            .ok_or_else(|| format_err!("Can not find block by hash {:?}", head_block_hash))?;
        Self::new_with_uncles(time_service, head, None, storage, vm_metrics)
    }

    fn new_with_uncles(
        time_service: Arc<dyn TimeService>,
        head_block: Block,
        uncles: Option<HashMap<HashValue, MintedUncleNumber>>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let block_info = storage
            .get_block_info(head_block.id())?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", head_block.id()))?;
        debug!("Init chain with block_info: {:?}", block_info);
        let state_root = head_block.header().state_root();
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let chain_state = ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root));
        let epoch = get_epoch_from_statedb(&chain_state)?;
        let genesis = storage
            .get_genesis()?
            .ok_or_else(|| format_err!("Can not find genesis hash in storage."))?;
        watch(CHAIN_WATCH_NAME, "n1253");
        let mut chain = Self {
            genesis_hash: genesis,
            time_service,
            txn_accumulator: info_2_accumulator(
                txn_accumulator_info.clone(),
                AccumulatorStoreType::Transaction,
                storage.as_ref(),
            ),
            block_accumulator: info_2_accumulator(
                block_accumulator_info.clone(),
                AccumulatorStoreType::Block,
                storage.as_ref(),
            ),
            status: ChainStatusWithBlock {
                status: ChainStatus::new(head_block.header.clone(), block_info),
                head: head_block,
            },
            statedb: chain_state,
            storage,
            uncles: HashMap::new(),
            epoch,
            vm_metrics,
        };
        watch(CHAIN_WATCH_NAME, "n1251");
        match uncles {
            Some(data) => chain.uncles = data,
            None => chain.update_uncle_cache()?,
        }
        watch(CHAIN_WATCH_NAME, "n1252");
        Ok(chain)
    }

    pub fn new_with_genesis(
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        genesis_epoch: Epoch,
        genesis_block: Block,
    ) -> Result<Self> {
        debug_assert!(genesis_block.header().is_genesis());
        let txn_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );
        let block_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Block),
        );
        let statedb = ChainStateDB::new(storage.clone().into_super_arc(), None);
        let executed_block = Self::execute_block_and_save(
            storage.as_ref(),
            statedb,
            txn_accumulator,
            block_accumulator,
            &genesis_epoch,
            None,
            genesis_block,
            None,
        )?;
        Self::new(time_service, executed_block.block.id(), storage, None)
    }

    pub fn current_epoch_uncles_size(&self) -> u64 {
        self.uncles.len() as u64
    }

    pub fn current_block_accumulator_info(&self) -> AccumulatorInfo {
        self.block_accumulator.get_info()
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        self.epoch.strategy()
    }
    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }

    //TODO lazy init uncles cache.
    fn update_uncle_cache(&mut self) -> Result<()> {
        self.uncles = self.epoch_uncles()?;
        Ok(())
    }

    fn epoch_uncles(&self) -> Result<HashMap<HashValue, MintedUncleNumber>> {
        let epoch = &self.epoch;
        let mut uncles: HashMap<HashValue, MintedUncleNumber> = HashMap::new();
        let head_block = self.head_block().block;
        let head_number = head_block.header().number();
        if head_number < epoch.start_block_number() || head_number >= epoch.end_block_number() {
            return Err(format_err!(
                "head block {} not in current epoch: {:?}.",
                head_number,
                epoch
            ));
        }
        for block_number in epoch.start_block_number()..epoch.end_block_number() {
            let block_uncles = if block_number == head_number {
                head_block.uncle_ids()
            } else {
                self.get_block_by_number(block_number)?
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find block by number {}, head block number: {}",
                            block_number,
                            head_number
                        )
                    })?
                    .uncle_ids()
            };
            block_uncles.into_iter().for_each(|uncle_id| {
                uncles.insert(uncle_id, block_number);
            });
            if block_number == head_number {
                break;
            }
        }

        Ok(uncles)
    }

    pub fn create_block_template(
        &self,
        author: AccountAddress,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        //FIXME create block template by parent may be use invalid chain state, such as epoch.
        //So the right way should be creating a BlockChain by parent_hash, then create block template.
        //the timestamp should be an argument, if want to mock an early block.
        let previous_header = match parent_hash {
            Some(hash) => self
                .get_header(hash)?
                .ok_or_else(|| format_err!("Can find block header by {:?}", hash))?,
            None => self.current_header(),
        };

        self.create_block_template_inner(
            author,
            previous_header,
            user_txns,
            uncles,
            block_gas_limit,
        )
    }

    fn create_block_template_inner(
        &self,
        author: AccountAddress,
        previous_header: BlockHeader,
        user_txns: Vec<SignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        let epoch = self.epoch();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let final_block_gas_limit = block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        let strategy = epoch.strategy();
        let difficulty = strategy.calculate_next_difficulty(self)?;
        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header,
            final_block_gas_limit,
            author,
            self.time_service.now_millis(),
            uncles,
            difficulty,
            strategy,
            None,
        )?;
        let excluded_txns = opened_block.push_txns(user_txns)?;
        let template = opened_block.finalize()?;
        Ok((template, excluded_txns))
    }

    /// Get block hash by block number, if not exist, return Error.
    pub fn get_hash_by_number_ensure(&self, number: BlockNumber) -> Result<HashValue> {
        self.get_hash_by_number(number)?
            .ok_or_else(|| format_err!("Can not find block hash by number {}", number))
    }

    fn check_exist_block(&self, block_id: HashValue, block_number: BlockNumber) -> Result<bool> {
        Ok(self
            .get_hash_by_number(block_number)?
            .filter(|hash| hash == &block_id)
            .is_some())
    }

    // filter block by check exist
    fn exist_block_filter(&self, block: Option<Block>) -> Result<Option<Block>> {
        Ok(match block {
            Some(block) => {
                if self.check_exist_block(block.id(), block.header().number())? {
                    Some(block)
                } else {
                    None
                }
            }
            None => None,
        })
    }

    // filter header by check exist
    fn exist_header_filter(&self, header: Option<BlockHeader>) -> Result<Option<BlockHeader>> {
        Ok(match header {
            Some(header) => {
                if self.check_exist_block(header.id(), header.number())? {
                    Some(header)
                } else {
                    None
                }
            }
            None => None,
        })
    }

    pub fn get_storage(&self) -> Arc<dyn Store> {
        self.storage.clone()
    }

    pub fn can_be_uncle(&self, block_header: &BlockHeader) -> Result<bool> {
        FullVerifier::can_be_uncle(self, block_header)
    }

    pub fn verify_with_verifier<V>(&mut self, block: Block) -> Result<VerifiedBlock>
    where
        V: BlockVerifier,
    {
        V::verify_block(self, block)
    }

    pub fn apply_with_verifier<V>(&mut self, block: Block) -> Result<ExecutedBlock>
    where
        V: BlockVerifier,
    {
        let verified_block = self.verify_with_verifier::<V>(block)?;
        watch(CHAIN_WATCH_NAME, "n1");
        let executed_block = self.execute(verified_block)?;
        watch(CHAIN_WATCH_NAME, "n2");
        self.connect(executed_block)
    }

    pub fn verify_without_save<V>(&mut self, block: Block) -> Result<ExecutedBlock>
    where
        V: BlockVerifier,
    {
        let verified_block = self.verify_with_verifier::<V>(block)?;
        watch(CHAIN_WATCH_NAME, "n1");
        self.execute_without_save(verified_block)
    }

    //TODO remove this function.
    pub fn update_chain_head(&mut self, block: Block) -> Result<ExecutedBlock> {
        let block_info = self
            .storage
            .get_block_info(block.id())?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", block.id()))?;
        self.connect(ExecutedBlock { block, block_info })
    }

    //TODO consider move this logic to BlockExecutor
    fn execute_block_and_save(
        storage: &dyn Store,
        statedb: ChainStateDB,
        txn_accumulator: MerkleAccumulator,
        block_accumulator: MerkleAccumulator,
        epoch: &Epoch,
        parent_status: Option<ChainStatus>,
        block: Block,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<ExecutedBlock> {
        let header = block.header();
        debug_assert!(header.is_genesis() || parent_status.is_some());
        debug_assert!(!header.is_genesis() || parent_status.is_none());
        let block_id = header.id();
        let transactions = {
            // genesis block do not generate BlockMetadata transaction.
            let mut t = match &parent_status {
                None => vec![],
                Some(parent) => {
                    let block_metadata = block.to_metadata(parent.head().gas_used());
                    vec![Transaction::BlockMetadata(block_metadata)]
                }
            };
            t.extend(
                block
                    .transactions()
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            t
        };

        watch(CHAIN_WATCH_NAME, "n21");
        let executed_data = starcoin_executor::block_execute(
            &statedb,
            transactions.clone(),
            epoch.block_gas_limit(),
            vm_metrics,
        )?;
        watch(CHAIN_WATCH_NAME, "n22");
        let state_root = executed_data.state_root;
        let vec_transaction_info = &executed_data.txn_infos;
        verify_block!(
            VerifyBlockField::State,
            state_root == header.state_root(),
            "verify block:{:?} state_root fail",
            block_id,
        );
        let block_gas_used = vec_transaction_info
            .iter()
            .fold(0u64, |acc, i| acc.saturating_add(i.gas_used()));
        verify_block!(
            VerifyBlockField::State,
            block_gas_used == header.gas_used(),
            "invalid block: gas_used is not match"
        );

        verify_block!(
            VerifyBlockField::State,
            vec_transaction_info.len() == transactions.len(),
            "invalid txn num in the block"
        );

        let transaction_global_index = txn_accumulator.num_leaves();

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
        );

        watch(CHAIN_WATCH_NAME, "n23");
        statedb
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        // If chain state is matched, and accumulator is matched,
        // then, we save flush states, and save block data.
        watch(CHAIN_WATCH_NAME, "n24");
        txn_accumulator
            .flush()
            .map_err(|_err| BlockExecutorError::BlockAccumulatorFlushErr)?;

        let pre_total_difficulty = parent_status
            .map(|status| status.total_difficulty())
            .unwrap_or_default();

        let total_difficulty = pre_total_difficulty + header.difficulty();

        block_accumulator.append(&[block_id])?;
        block_accumulator.flush()?;

        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        let block_info = BlockInfo::new(
            block_id,
            total_difficulty,
            txn_accumulator_info,
            block_accumulator_info,
        );

        watch(CHAIN_WATCH_NAME, "n25");

        // save block's transaction relationship and save transaction

        let block_id = block.id();
        let txn_infos = executed_data.txn_infos;
        let txn_events = executed_data.txn_events;
        let txn_table_infos = executed_data
            .txn_table_infos
            .into_iter()
            .collect::<Vec<_>>();

        debug_assert!(
            txn_events.len() == txn_infos.len(),
            "events' length should be equal to txn infos' length"
        );
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
            storage.save_contract_events(*info_id, events)?;
        }

        storage.save_transaction_infos(
            txn_infos
                .into_iter()
                .enumerate()
                .map(|(transaction_index, info)| {
                    RichTransactionInfo::new(
                        block_id,
                        block.header().number(),
                        info,
                        transaction_index as u32,
                        transaction_global_index
                            .checked_add(transaction_index as u64)
                            .expect("transaction_global_index overflow."),
                    )
                })
                .collect(),
        )?;

        let txn_id_vec = transactions
            .iter()
            .map(|user_txn| user_txn.id())
            .collect::<Vec<HashValue>>();
        // save transactions
        storage.save_transaction_batch(transactions)?;

        // save block's transactions
        storage.save_block_transaction_ids(block_id, txn_id_vec)?;
        storage.save_block_txn_info_ids(block_id, txn_info_ids)?;
        storage.commit_block(block.clone())?;

        storage.save_block_info(block_info.clone())?;

        storage.save_table_infos(txn_table_infos)?;

        watch(CHAIN_WATCH_NAME, "n26");
        Ok(ExecutedBlock { block, block_info })
    }

    fn execute_save_directly(
        storage: &dyn Store,
        statedb: ChainStateDB,
        txn_accumulator: MerkleAccumulator,
        block_accumulator: MerkleAccumulator,
        parent_status: Option<ChainStatus>,
        block: Block,
        block_info: BlockInfo,
        executed_data: BlockExecutedData,
    ) -> Result<ExecutedBlock> {
        let header = block.header();
        let block_id = header.id();

        let transactions = {
            // genesis block do not generate BlockMetadata transaction.
            let mut t = match &parent_status {
                None => vec![],
                Some(parent) => {
                    let block_metadata = block.to_metadata(parent.head().gas_used());
                    vec![Transaction::BlockMetadata(block_metadata)]
                }
            };
            t.extend(
                block
                    .transactions()
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            t
        };
        for write_set in executed_data.write_sets {
            statedb
                .apply_write_set(write_set)
                .map_err(BlockExecutorError::BlockChainStateErr)?;
            statedb
                .commit()
                .map_err(BlockExecutorError::BlockChainStateErr)?;
        }
        let vec_transaction_info = &executed_data.txn_infos;
        verify_block!(
            VerifyBlockField::State,
            statedb.state_root() == header.state_root(),
            "verify block:{:?} state_root fail",
            block_id,
        );

        verify_block!(
            VerifyBlockField::State,
            vec_transaction_info.len() == transactions.len(),
            "invalid txn num in the block"
        );

        let transaction_global_index = txn_accumulator.num_leaves();

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
        );

        statedb
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        // If chain state is matched, and accumulator is matched,
        // then, we save flush states, and save block data.
        txn_accumulator
            .flush()
            .map_err(|_err| BlockExecutorError::BlockAccumulatorFlushErr)?;

        block_accumulator.append(&[block_id])?;
        block_accumulator.flush()?;

        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        verify_block!(
            VerifyBlockField::State,
            txn_accumulator_info == block_info.txn_accumulator_info,
            "verify block: txn accumulator info mismatch"
        );

        verify_block!(
            VerifyBlockField::State,
            block_accumulator_info == block_info.block_accumulator_info,
            "verify block: block accumulator info mismatch"
        );

        let txn_infos = executed_data.txn_infos;
        let txn_events = executed_data.txn_events;
        let txn_table_infos = executed_data
            .txn_table_infos
            .into_iter()
            .collect::<Vec<_>>();

        // save block's transaction relationship and save transaction
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
            storage.save_contract_events(*info_id, events)?;
        }

        storage.save_transaction_infos(
            txn_infos
                .into_iter()
                .enumerate()
                .map(|(transaction_index, info)| {
                    RichTransactionInfo::new(
                        block_id,
                        block.header().number(),
                        info,
                        transaction_index as u32,
                        transaction_global_index
                            .checked_add(transaction_index as u64)
                            .expect("transaction_global_index overflow."),
                    )
                })
                .collect(),
        )?;

        let txn_id_vec = transactions
            .iter()
            .map(|user_txn| user_txn.id())
            .collect::<Vec<HashValue>>();
        // save transactions
        storage.save_transaction_batch(transactions)?;

        // save block's transactions
        storage.save_block_transaction_ids(block_id, txn_id_vec)?;
        storage.save_block_txn_info_ids(block_id, txn_info_ids)?;
        storage.commit_block(block.clone())?;

        storage.save_block_info(block_info.clone())?;

        storage.save_table_infos(txn_table_infos)?;

        Ok(ExecutedBlock { block, block_info })
    }

    pub fn set_output_block() {
        OUTPUT_BLOCK.store(true, Ordering::Relaxed);
    }

    fn execute_block_without_save(
        statedb: ChainStateDB,
        txn_accumulator: MerkleAccumulator,
        block_accumulator: MerkleAccumulator,
        epoch: &Epoch,
        parent_status: Option<ChainStatus>,
        block: Block,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<ExecutedBlock> {
        let header = block.header();
        debug_assert!(header.is_genesis() || parent_status.is_some());
        debug_assert!(!header.is_genesis() || parent_status.is_none());
        let block_id = header.id();
        let transactions = {
            // genesis block do not generate BlockMetadata transaction.
            let mut t = match &parent_status {
                None => vec![],
                Some(parent) => {
                    let block_metadata = block.to_metadata(parent.head().gas_used());
                    vec![Transaction::BlockMetadata(block_metadata)]
                }
            };
            t.extend(
                block
                    .transactions()
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            t
        };

        watch(CHAIN_WATCH_NAME, "n21");
        let executed_data = starcoin_executor::block_execute(
            &statedb,
            transactions.clone(),
            epoch.block_gas_limit(),
            vm_metrics,
        )?;
        watch(CHAIN_WATCH_NAME, "n22");
        let state_root = executed_data.state_root;
        let vec_transaction_info = &executed_data.txn_infos;
        verify_block!(
            VerifyBlockField::State,
            state_root == header.state_root(),
            "verify block:{:?} state_root fail",
            block_id,
        );
        let block_gas_used = vec_transaction_info
            .iter()
            .fold(0u64, |acc, i| acc.saturating_add(i.gas_used()));
        verify_block!(
            VerifyBlockField::State,
            block_gas_used == header.gas_used(),
            "invalid block: gas_used is not match"
        );

        verify_block!(
            VerifyBlockField::State,
            vec_transaction_info.len() == transactions.len(),
            "invalid txn num in the block"
        );

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
        );

        let pre_total_difficulty = parent_status
            .map(|status| status.total_difficulty())
            .unwrap_or_default();
        let total_difficulty = pre_total_difficulty + header.difficulty();
        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        let block_info = BlockInfo::new(
            block_id,
            total_difficulty,
            txn_accumulator_info,
            block_accumulator_info,
        );

        watch(CHAIN_WATCH_NAME, "n25");

        debug_assert!(
            executed_data.txn_events.len() == executed_data.txn_infos.len(),
            "events' length should be equal to txn infos' length"
        );
        if OUTPUT_BLOCK.load(Ordering::Relaxed) {
            println!("// {}", block.header().number());
            println!("maps.insert(");
            println!("HashValue::from_hex_literal(\"{}\").unwrap(),",block.id());
            println!("(\nserde_json::from_str({:?}).unwrap(),",serde_json::to_string(&executed_data)?);
            println!("\nserde_json::from_str({:?}).unwrap()\n)\n);",serde_json::to_string(&block_info)?);
        }

        Ok(ExecutedBlock { block, block_info })
    }

    pub fn get_txn_accumulator(&self) -> &MerkleAccumulator {
        &self.txn_accumulator
    }

    pub fn get_block_accumulator(&self) -> &MerkleAccumulator {
        &self.block_accumulator
    }
}

impl ChainReader for BlockChain {
    fn info(&self) -> ChainInfo {
        ChainInfo::new(
            self.status.head.header().chain_id(),
            self.genesis_hash,
            self.status.status.clone(),
        )
    }

    fn status(&self) -> ChainStatus {
        self.status.status.clone()
    }

    fn head_block(&self) -> ExecutedBlock {
        ExecutedBlock::new(self.status.head.clone(), self.status.status.info.clone())
    }

    fn current_header(&self) -> BlockHeader {
        self.status.status.head().clone()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage
            .get_block_header_by_hash(hash)
            .and_then(|block_header| self.exist_header_filter(block_header))
    }

    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        self.get_hash_by_number(number)
            .and_then(|block_id| match block_id {
                None => Ok(None),
                Some(block_id) => self.storage.get_block_header_by_hash(block_id),
            })
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        self.get_hash_by_number(number)
            .and_then(|block_id| match block_id {
                None => Ok(None),
                Some(block_id) => self.storage.get_block_by_hash(block_id),
            })
    }

    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        reverse: bool,
        count: u64,
    ) -> Result<Vec<Block>> {
        let end_num = match number {
            None => self.current_header().number(),
            Some(number) => number,
        };

        let num_leaves = self.block_accumulator.num_leaves();

        if end_num > num_leaves.saturating_sub(1) {
            bail!("Can not find block by number {}", end_num);
        };

        let len = if !reverse && (end_num.saturating_add(count) > num_leaves.saturating_sub(1)) {
            num_leaves.saturating_sub(end_num)
        } else {
            count
        };

        let ids = self.get_block_ids(end_num, reverse, len)?;
        let block_opts = self.storage.get_blocks(ids)?;
        let mut blocks = vec![];
        for (idx, block) in block_opts.into_iter().enumerate() {
            match block {
                Some(block) => blocks.push(block),
                None => bail!(
                    "Can not find block by number {}",
                    end_num.saturating_sub(idx as u64)
                ),
            }
        }
        Ok(blocks)
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage
            .get_block_by_hash(hash)
            .and_then(|block| self.exist_block_filter(block))
    }

    fn get_hash_by_number(&self, number: BlockNumber) -> Result<Option<HashValue>> {
        self.block_accumulator.get_leaf(number)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
        //TODO check txn should exist on current chain.
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<RichTransactionInfo>> {
        let txn_info_ids = self
            .storage
            .get_transaction_info_ids_by_txn_hash(txn_hash)?;
        for txn_info_id in txn_info_ids {
            let txn_info = self.storage.get_transaction_info(txn_info_id)?;
            if let Some(txn_info) = txn_info {
                if self.exist_block(txn_info.block_id())? {
                    return Ok(Some(txn_info));
                }
            }
        }
        Ok(None)
    }

    fn get_transaction_info_by_global_index(
        &self,
        transaction_global_index: u64,
    ) -> Result<Option<RichTransactionInfo>> {
        match self.txn_accumulator.get_leaf(transaction_global_index)? {
            None => Ok(None),
            Some(hash) => self.storage.get_transaction_info(hash),
        }
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        &self.statedb
    }

    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>> {
        match block_id {
            Some(block_id) => self.storage.get_block_info(block_id),
            None => Ok(Some(self.status.status.info().clone())),
        }
    }

    fn get_total_difficulty(&self) -> Result<U256> {
        Ok(self.status.status.total_difficulty())
    }

    fn exist_block(&self, block_id: HashValue) -> Result<bool> {
        if let Some(header) = self.storage.get_block_header_by_hash(block_id)? {
            return self.check_exist_block(block_id, header.number());
        }
        Ok(false)
    }

    fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>> {
        self.block_accumulator
            .get_leaves(start_number, reverse, max_size)
    }

    fn get_block_info_by_number(&self, number: BlockNumber) -> Result<Option<BlockInfo>> {
        let block = self
            .get_block_by_number(number)?
            .ok_or_else(|| format_err!("Can not find block by number {}", number))?;

        self.get_block_info(Some(block.id()))
    }

    fn time_service(&self) -> &dyn TimeService {
        self.time_service.as_ref()
    }

    fn fork(&self, block_id: HashValue) -> Result<Self> {
        ensure!(
            self.exist_block(block_id)?,
            "Block with id{} do not exists in current chain.",
            block_id
        );
        let head = self
            .storage
            .get_block_by_hash(block_id)?
            .ok_or_else(|| format_err!("Can not find block by hash {:?}", block_id))?;
        // if fork block_id is at same epoch, try to reuse uncles cache.
        let uncles = if head.header().number() >= self.epoch.start_block_number() {
            Some(
                self.uncles
                    .iter()
                    .filter(|(_uncle_id, uncle_number)| **uncle_number <= head.header().number())
                    .map(|(uncle_id, uncle_number)| (*uncle_id, *uncle_number))
                    .collect::<HashMap<HashValue, MintedUncleNumber>>(),
            )
        } else {
            None
        };
        BlockChain::new_with_uncles(
            self.time_service.clone(),
            head,
            uncles,
            self.storage.clone(),
            self.vm_metrics.clone(),
        )
    }

    fn epoch_uncles(&self) -> &HashMap<HashValue, MintedUncleNumber> {
        &self.uncles
    }

    fn find_ancestor(&self, another: &dyn ChainReader) -> Result<Option<BlockIdAndNumber>> {
        let other_header_number = another.current_header().number();
        let self_header_number = self.current_header().number();
        let min_number = std::cmp::min(other_header_number, self_header_number);
        let mut ancestor = None;
        for block_number in (0..min_number).rev() {
            let block_id_1 = another.get_hash_by_number(block_number)?;
            let block_id_2 = self.get_hash_by_number(block_number)?;
            match (block_id_1, block_id_2) {
                (Some(block_id_1), Some(block_id_2)) => {
                    if block_id_1 == block_id_2 {
                        ancestor = Some(BlockIdAndNumber::new(block_id_1, block_number));
                        break;
                    }
                }
                (_, _) => {
                    continue;
                }
            }
        }
        Ok(ancestor)
    }

    fn verify(&self, block: Block) -> Result<VerifiedBlock> {
        FullVerifier::verify_block(self, block)
    }

    fn execute(&self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        if let Some((executed_data, block_info)) =
            MAIN_DIRECT_SAVE_BLOCK_HASH_MAP.get(&verified_block.0.header.id())
        {
            Self::execute_save_directly(
                self.storage.as_ref(),
                self.statedb.fork(),
                self.txn_accumulator.fork(None),
                self.block_accumulator.fork(None),
                Some(self.status.status.clone()),
                verified_block.0,
                block_info.clone(),
                executed_data.clone(),
            )
        } else {
            Self::execute_block_and_save(
                self.storage.as_ref(),
                self.statedb.fork(),
                self.txn_accumulator.fork(None),
                self.block_accumulator.fork(None),
                &self.epoch,
                Some(self.status.status.clone()),
                verified_block.0,
                self.vm_metrics.clone(),
            )
        }
    }

    fn execute_without_save(&self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        Self::execute_block_without_save(
            self.statedb.fork(),
            self.txn_accumulator.fork(None),
            self.block_accumulator.fork(None),
            &self.epoch,
            Some(self.status.status.clone()),
            verified_block.0,
            self.vm_metrics.clone(),
        )
    }

    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>> {
        let chain_header = self.current_header();
        let hashes = self
            .txn_accumulator
            .get_leaves(start_index, reverse, max_size)?;
        let mut infos = vec![];
        let txn_infos = self.storage.get_transaction_infos(hashes.clone())?;
        for (i, info) in txn_infos.into_iter().enumerate() {
            match info {
                Some(info) => infos.push(info),
                None => bail!(
                    "cannot find hash({:?}) on head: {}",
                    hashes.get(i),
                    chain_header.id()
                ),
            }
        }
        Ok(infos)
    }

    fn get_events(&self, txn_info_id: HashValue) -> Result<Option<Vec<ContractEvent>>> {
        self.storage.get_contract_events(txn_info_id)
    }

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>> {
        let block_info = match self.get_block_info(Some(block_id))? {
            Some(block_info) => block_info,
            None => return Ok(None),
        };
        let accumulator = self
            .txn_accumulator
            .fork(Some(block_info.txn_accumulator_info));
        let txn_proof = match accumulator.get_proof(transaction_global_index)? {
            Some(proof) => proof,
            None => return Ok(None),
        };

        //if can get proof by leaf_index, the leaf and transaction info should exist.
        let txn_info_hash = self
            .txn_accumulator
            .get_leaf(transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "Can not find txn info hash by index {}",
                    transaction_global_index
                )
            })?;
        let transaction_info = self
            .storage
            .get_transaction_info(txn_info_hash)?
            .ok_or_else(|| format_err!("Can not find txn info by hash:{}", txn_info_hash))?;

        let event_proof = if let Some(event_index) = event_index {
            let events = self
                .storage
                .get_contract_events(txn_info_hash)?
                .unwrap_or_default();
            let event = events.get(event_index as usize).cloned().ok_or_else(|| {
                format_err!("event index out of range, events len:{}", events.len())
            })?;
            let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();

            let event_proof =
                InMemoryAccumulator::get_proof_from_leaves(event_hashes.as_slice(), event_index)?;
            Some(EventWithProof {
                event,
                proof: event_proof,
            })
        } else {
            None
        };
        let state_proof = if let Some(access_path) = access_path {
            let statedb = self
                .statedb
                .fork_at(transaction_info.txn_info().state_root_hash());
            Some(statedb.get_with_proof(&access_path)?)
        } else {
            None
        };
        Ok(Some(TransactionInfoWithProof {
            transaction_info,
            proof: txn_proof,
            event_proof,
            state_proof,
        }))
    }
}

impl BlockChain {
    pub fn filter_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>> {
        let reverse = filter.reverse;
        let chain_header = self.current_header();
        let max_block_number = chain_header.number().min(filter.to_block);

        // quick return.
        if filter.from_block > max_block_number {
            return Ok(vec![]);
        }

        let (mut cur_block_number, tail) = if reverse {
            (max_block_number, filter.from_block)
        } else {
            (filter.from_block, max_block_number)
        };
        let mut event_with_infos = vec![];
        'outer: loop {
            let block = self.get_block_by_number(cur_block_number)?.ok_or_else(|| {
                anyhow::anyhow!(format!(
                    "cannot find block({}) on main chain(head: {})",
                    cur_block_number,
                    chain_header.id()
                ))
            })?;
            let block_id = block.id();
            let block_number = block.header().number();
            let mut txn_info_ids = self.storage.get_block_txn_info_ids(block_id)?;
            if reverse {
                txn_info_ids.reverse();
            }
            for id in txn_info_ids.iter() {
                let events = self.storage.get_contract_events(*id)?.ok_or_else(|| {
                    anyhow::anyhow!(format!(
                        "cannot find events of txn with txn_info_id {} on main chain(header: {})",
                        id,
                        chain_header.id()
                    ))
                })?;
                let mut filtered_events = events
                    .into_iter()
                    .enumerate()
                    .filter(|(_idx, evt)| filter.matching(block_number, evt))
                    .peekable();
                if filtered_events.peek().is_none() {
                    continue;
                }

                let txn_info = self.storage.get_transaction_info(*id)?.ok_or_else(|| {
                    anyhow::anyhow!(format!(
                        "cannot find txn info with txn_info_id {} on main chain(head: {})",
                        id,
                        chain_header.id()
                    ))
                })?;

                let filtered_event_with_info =
                    filtered_events.map(|(idx, evt)| ContractEventInfo {
                        block_hash: block_id,
                        block_number: block.header().number(),
                        transaction_hash: txn_info.transaction_hash(),
                        transaction_index: txn_info.transaction_index,
                        transaction_global_index: txn_info.transaction_global_index,
                        event_index: idx as u32,
                        event: evt,
                    });
                if reverse {
                    event_with_infos.extend(filtered_event_with_info.rev())
                } else {
                    event_with_infos.extend(filtered_event_with_info);
                }

                if let Some(limit) = filter.limit {
                    if event_with_infos.len() >= limit {
                        break 'outer;
                    }
                }
            }

            let should_break = match reverse {
                true => cur_block_number <= tail,
                false => cur_block_number >= tail,
            };

            if should_break {
                break 'outer;
            }

            if reverse {
                cur_block_number = cur_block_number.saturating_sub(1);
            } else {
                cur_block_number = cur_block_number.saturating_add(1);
            }
        }

        // remove additional events in respect limit filter.
        if let Some(limit) = filter.limit {
            event_with_infos.truncate(limit);
        }
        Ok(event_with_infos)
    }
}

impl ChainWriter for BlockChain {
    fn can_connect(&self, executed_block: &ExecutedBlock) -> bool {
        executed_block.block.header().parent_hash() == self.status.status.head().id()
    }

    fn connect(&mut self, executed_block: ExecutedBlock) -> Result<ExecutedBlock> {
        let (block, block_info) = (executed_block.block(), executed_block.block_info());
        debug_assert!(block.header().parent_hash() == self.status.status.head().id());
        //TODO try reuse accumulator and state db.
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let state_root = block.header().state_root();
        self.txn_accumulator = info_2_accumulator(
            txn_accumulator_info.clone(),
            AccumulatorStoreType::Transaction,
            self.storage.as_ref(),
        );
        self.block_accumulator = info_2_accumulator(
            block_accumulator_info.clone(),
            AccumulatorStoreType::Block,
            self.storage.as_ref(),
        );

        self.statedb = ChainStateDB::new(self.storage.clone().into_super_arc(), Some(state_root));
        self.status = ChainStatusWithBlock {
            status: ChainStatus::new(block.header().clone(), block_info.clone()),
            head: block.clone(),
        };
        if self.epoch.end_block_number() == block.header().number() {
            self.epoch = get_epoch_from_statedb(&self.statedb)?;
            self.update_uncle_cache()?;
        } else if let Some(block_uncles) = block.uncles() {
            block_uncles.iter().for_each(|uncle_header| {
                self.uncles
                    .insert(uncle_header.id(), block.header().number());
            });
        }
        Ok(executed_block)
    }

    fn apply(&mut self, block: Block) -> Result<ExecutedBlock> {
        self.apply_with_verifier::<FullVerifier>(block)
    }

    fn chain_state(&mut self) -> &ChainStateDB {
        &self.statedb
    }
}

pub(crate) fn info_2_accumulator(
    accumulator_info: AccumulatorInfo,
    store_type: AccumulatorStoreType,
    node_store: &dyn Store,
) -> MerkleAccumulator {
    MerkleAccumulator::new_with_info(
        accumulator_info,
        node_store.get_accumulator_store(store_type),
    )
}

fn get_epoch_from_statedb(statedb: &ChainStateDB) -> Result<Epoch> {
    let account_reader = AccountStateReader::new(statedb);
    account_reader
        .get_resource::<Epoch>(genesis_address())?
        .ok_or_else(|| format_err!("Epoch is none."))
}
