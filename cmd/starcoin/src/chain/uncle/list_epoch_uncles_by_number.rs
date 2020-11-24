use crate::cli_state::CliState;
use crate::view::{BlockSimpleView, UncleInfo};
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use std::collections::HashMap;
use structopt::StructOpt;

/// Show block list which uncles is some in a epoch.
#[derive(Debug, StructOpt)]
#[structopt(name = "list_epoch_uncles_by_number")]
pub struct ListEpochUnclesByNumberOpt {
    #[structopt(name = "number", long, short = "n", default_value = "0")]
    number: u64,
}

pub struct ListEpochUnclesByNumberCommand;

impl CommandAction for ListEpochUnclesByNumberCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListEpochUnclesByNumberOpt;
    type ReturnItem = Vec<UncleInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let block_summaries = client.get_epoch_uncles_by_number(ctx.opt().number)?;
        let mut ids = Vec::new();
        block_summaries.iter().for_each(|block_summary| {
            if let Some(uncles) = block_summary.uncles() {
                uncles.iter().for_each(|uncle_header| {
                    ids.push(uncle_header.parent_hash());
                });
            }
        });

        let mut header_map = HashMap::new();
        let parent_headers = client.get_headers(ids)?;
        parent_headers.into_iter().for_each(|header| {
            header_map.insert(header.id(), header);
        });

        let mut views = Vec::new();
        block_summaries.into_iter().for_each(|block_summary| {
            if block_summary.uncles().is_some() {
                let (block_header, uncles) = block_summary.into();
                let block_view: BlockSimpleView = block_header.into();
                uncles.unwrap().into_iter().for_each(|uncle_header| {
                    if let Some(parent_header) = header_map.get(&uncle_header.parent_hash()) {
                        let uncle_view: BlockSimpleView = uncle_header.into();
                        let uncle_parent_view: BlockSimpleView = parent_header.clone().into();
                        let uncle_info = UncleInfo::new(uncle_view, uncle_parent_view, block_view);
                        views.push(uncle_info);
                    }
                });
            }
        });

        Ok(views)
    }
}
