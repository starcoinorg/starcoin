// SPDX-License-Identifier: Apache-2.0
// Copyright (c) The Starcoin Core Contributors

use crate::view::{ExecuteResultView, TransactionOptions};
use crate::{CliState, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_transaction_builder::encode_nft_transfer_script;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::on_chain_resource::nft::{IdentifierNFT, NFTGallery, NFT, NFTUUID};
use starcoin_vm_types::transaction::TransactionPayload;

/// Some commands for nft.
#[derive(Debug, Parser)]
#[clap(name = "nft")]
#[allow(clippy::large_enum_variant)]
#[allow(clippy::upper_case_acronyms)]
pub enum NFTOpt {
    /// List all NFT in the NFTGallery of the account
    #[clap(name = "list")]
    List {
        #[clap(name = "address")]
        /// The account's address to list, if absent, show the default account.
        address: Option<AccountAddress>,
    },
    /// List all IdentifierNFT of the account
    #[clap(name = "ident", alias = "identifier")]
    Identifier {
        #[clap(name = "address")]
        /// The account's address to show, if absent, show the default account.
        address: Option<AccountAddress>,
    },
    /// Transfer NFT to `receiver`
    #[clap(name = "transfer")]
    Transfer {
        #[clap(long = "uuid")]
        uuid: NFTUUID,
        #[clap(short = 'r', long = "receiver")]
        receiver: AccountAddress,
        #[clap(flatten)]
        transaction_opts: TransactionOptions,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct NFTView {
    pub uuid: NFTUUID,
    #[serde(flatten)]
    pub nft: NFT,
}

impl From<NFT> for NFTView {
    fn from(nft: NFT) -> Self {
        let uuid = nft.uuid();
        NFTView { uuid, nft }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::large_enum_variant)]
pub enum NFTResult {
    List(Vec<NFTView>),
    Ident(Vec<IdentifierNFT>),
    Transfer(ExecuteResultView),
}

#[allow(clippy::upper_case_acronyms)]
pub struct NFTCommand;

impl CommandAction for NFTCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = NFTOpt;
    type ReturnItem = NFTResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let cmd_result = match opt {
            NFTOpt::List { address } => {
                let address = ctx
                    .state()
                    .get_account_or_default(*address)
                    .map(|account| account.address)?;
                let all_resources = ctx.state().client().state_list_resource(
                    address,
                    true,
                    None,
                    0,
                    usize::MAX,
                    None,
                )?;
                let galleries: Result<Vec<NFTGallery>> = all_resources
                    .resources
                    .into_iter()
                    .filter_map(|(resource_type, resource)| {
                        if let Some(nft_type) = NFTGallery::nft_type(&resource_type.0) {
                            Some(NFTGallery::from_json(
                                nft_type,
                                resource.json.expect("resource json should decoded.").0,
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                let nfts: Vec<NFTView> = galleries?
                    .into_iter()
                    .flat_map(|gallery| gallery.items)
                    .map(NFTView::from)
                    .collect();
                NFTResult::List(nfts)
            }
            NFTOpt::Identifier { address } => {
                let address = ctx
                    .state()
                    .get_account_or_default(*address)
                    .map(|account| account.address)?;

                let all_resources = ctx.state().client().state_list_resource(
                    address,
                    true,
                    None,
                    0,
                    usize::MAX,
                    None,
                )?;
                let ident_nfts: Result<Vec<IdentifierNFT>> = all_resources
                    .resources
                    .into_iter()
                    .filter_map(|(resource_type, resource)| {
                        if let Some(nft_type) = IdentifierNFT::nft_type(&resource_type.0) {
                            Some(IdentifierNFT::from_json(
                                nft_type,
                                resource.json.expect("resource json should decoded.").0,
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                NFTResult::Ident(ident_nfts?)
            }
            NFTOpt::Transfer {
                transaction_opts,
                uuid,
                receiver,
            } => {
                println!("{}", uuid);
                let script_function = encode_nft_transfer_script(uuid.clone(), *receiver);
                println!("{:?}", script_function);
                let result = ctx.state().build_and_execute_transaction(
                    transaction_opts.clone(),
                    TransactionPayload::ScriptFunction(script_function),
                );
                println!("{:?}", result);
                NFTResult::Transfer(result?)
            }
        };

        Ok(cmd_result)
    }
}
