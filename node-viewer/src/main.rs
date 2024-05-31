use std::path::PathBuf;
use eframe::NativeOptions;
use starcoin_config::BuiltinNetworkID;
use crate::dag_block_loader::load_blocks_from_db;

// mod dag_viewer;
mod dag_block_loader;
mod dagre_dag_viewer;

fn main() {
    let options = NativeOptions::default();
    // let nodes = vec![
    //     DagNode::new(
    //         "0x8c4be3996a1ca668502aa960bea74200a7a3f75f6a81882d662e2816ae15f7cb",
    //         &vec![],
    //         None,
    //         None,
    //         None,
    //     ),
    //     DagNode::new(
    //         "0x3234ef0518185a95b8afdfd8d84dd6db72938ece94bb376a9775d614f7957f7d",
    //         &vec!["0x8c4be3996a1ca668502aa960bea74200a7a3f75f6a81882d662e2816ae15f7cb".to_string()],
    //         None,
    //         None,
    //         None,
    //     ),
    //     DagNode::new(
    //         "0x6b4761f0377f47dcee8611c46fb5ed476194a5b1b0e0a4900ad81edccb0b7d45",
    //         &vec![
    //             "0x3234ef0518185a95b8afdfd8d84dd6db72938ece94bb376a9775d614f7957f7d".to_string(),
    //             "0x8c4be3996a1ca668502aa960bea74200a7a3f75f6a81882d662e2816ae15f7cb".to_string(),
    //         ],
    //         None,
    //         None,
    //         None,
    //     ),
    //     DagNode::new(
    //         "0x22d521063f085d5a4d3b2a17271e6d4059c0fbb7308f94b3c127df1de32bce9a",
    //         &vec!["0x6b4761f0377f47dcee8611c46fb5ed476194a5b1b0e0a4900ad81edccb0b7d45".to_string()],
    //         None,
    //         None,
    //         None,
    //     ),
    //     DagNode::new(
    //         "0xe98a4416bb8ac797e698d9669f3da3d2cb833438e32bc95d208c21607bc6e8e3",
    //         &vec!["0x8c4be3996a1ca668502aa960bea74200a7a3f75f6a81882d662e2816ae15f7cb".to_string()],
    //         None,
    //         None,
    //         None,
    //     ),
    //     //DagNode::new("B", &vec!["A".to_string()]),
    //     //DagNode::new("C", &vec!["A".to_string(), "B".to_string()]),
    //     //DagNode::new("D", &vec!["B".to_string(), "C".to_string()]),
    //     // DagNode::new("E", &vec!["D".to_string()]),
    //     // DagNode::new("F", &vec!["E".to_string()]),
    //     // DagNode::new("G", &vec!["F".to_string()]),
    //     // DagNode::new("H", &vec!["G".to_string()]),
    //     // DagNode::new("I", &vec!["H".to_string()]),
    //     // DagNode::new("J", &vec!["I".to_string()]),
    // ];
    let root_path = PathBuf::from("~/.starcoin/halley");
    let nodes = load_blocks_from_db(BuiltinNetworkID::Halley, None, None, &root_path).expect("Load node failed");
    let mut viewer = dagre_dag_viewer::DagViewer::new(nodes, 0.0, 15.0, 0.0, 0.0);
    viewer.set_need_layout();
    eframe::run_native("DAG Viewer", options, Box::new(|_cc| Box::new(viewer)))
        .expect("Failed to run");
}
