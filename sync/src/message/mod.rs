// use crate::download::DownloadActor;
// use crate::process::ProcessActor;
// use actix::prelude::*;
// use anyhow::Result;
// use crypto::HashValue;
// use std::cmp::Ordering;
// use types::{
//     block::{Block, BlockHeader},
//     peer_info::PeerInfo,
//     transaction::SignedUserTransaction,
// };
// use network::sync_messages::{
//     LatestStateMsg, BatchHashByNumberMsg, BatchHeaderMsg, BatchBodyMsg, GetHashByNumberMsg,
//     GetDataByHashMsg, BlockBody, DataType, HashWithNumber, HashWithBlockHeader,
// };
//
// #[derive(Message)]
// #[rtype(result = "()")]
// pub enum SyncMessage {
//     DownloadMessage(DownloadMessage),
//     ProcessMessage(ProcessMessage),
// }
//
// pub enum DownloadMessage {
//     LatestStateMsg(Option<Addr<ProcessActor>>, PeerInfo, LatestStateMsg),
//     BatchHashByNumberMsg(Option<Addr<ProcessActor>>, PeerInfo, BatchHashByNumberMsg),
//     BatchHeaderMsg(Option<Addr<ProcessActor>>, PeerInfo, BatchHeaderMsg),
//     BatchBodyMsg(Option<Addr<ProcessActor>>, BatchBodyMsg),
//     BatchHeaderAndBodyMsg(BatchHeaderMsg, BatchBodyMsg),
//     // just fo test
//     NewBlock(Block),
// }
//
// impl Message for DownloadMessage {
//     type Result = Result<()>;
// }
//
// pub enum ProcessMessage {
//     NewPeerMsg(PeerInfo),
//     GetHashByNumberMsg(Option<Addr<DownloadActor>>, GetHashByNumberMsg),
//     GetDataByHashMsg(Option<Addr<DownloadActor>>, GetDataByHashMsg),
// }
//
// impl Message for ProcessMessage {
//     type Result = Result<()>;
// }
