// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    view::TransactionOptions,
    view_vm2::{ExecuteResultView, ExecutionOutputView},
};

use anyhow::{anyhow, bail, format_err, Result};
use bcs_ext::BCSCodec;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use starcoin_config::{ChainNetworkID, DataDirPath};
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_vm2_abi_decoder::{decode_txn_payload, DecodedTransactionPayload};
use starcoin_vm2_account_api::{AccountInfo, AccountProvider};

use starcoin_logger::prelude::info;
use starcoin_rpc_api::chain::GetEventOption;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_crypto::{
    hash::PlainCryptoHash,
    multi_ed25519::{multi_shard::MultiEd25519SignatureShard, MultiEd25519PublicKey},
    HashValue,
};
use starcoin_vm2_dev::playground;
use starcoin_vm2_types::view::{
    DryRunOutputView, RawUserTransactionView, SignedUserTransactionView, TransactionPayloadView,
    TransactionStatusView,
};
use starcoin_vm2_vm_types::{
    account_address::AccountAddress,
    account_config::{association_address, AccountResource, STC_TOKEN_CODE_STR},
    genesis_config::ChainId,
    move_resource::MoveResource,
    state_view::StateReaderExt,
    transaction::{
        authenticator::{AccountPublicKey, TransactionAuthenticator},
        DryRunTransaction, RawUserTransaction, SignedUserTransaction, TransactionPayload,
    },
};
use std::env::current_dir;
use std::{fs::File, path::PathBuf, sync::Arc, time::Duration};

/// A reduced version of clistate, retaining only the necessary execution action functions
#[allow(dead_code)]
pub struct CliStateVM2 {
    net: ChainNetworkID,
    client: Arc<RpcClient>,
    account_client: Box<dyn AccountProvider>,
    watch_timeout: Duration,
    data_dir: PathBuf,
    temp_dir: DataDirPath,
}

fn build_dirs_from_net(net: &ChainNetworkID) -> Result<(PathBuf, DataDirPath)> {
    let data_dir = starcoin_config::G_DEFAULT_BASE_DATA_DIR
        .clone()
        .join("cli-vm2")
        .join(net.to_string());
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir.as_path())
            .unwrap_or_else(|e| panic!("Create cli data dir {:?} fail, err:{:?}", data_dir, e))
    }
    let temp_dir = data_dir.join("tmp-vm2");
    if !temp_dir.exists() {
        std::fs::create_dir_all(temp_dir.as_path())
            .unwrap_or_else(|e| panic!("Create cli temp dir {:?} fail, err:{:?}", temp_dir, e))
    }
    let temp_dir = starcoin_config::temp_dir_in(temp_dir);
    Ok((data_dir, temp_dir))
}

impl CliStateVM2 {
    pub const DEFAULT_WATCH_TIMEOUT: Duration = Duration::from_secs(300);
    pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 10000000;
    pub const DEFAULT_GAS_PRICE: u64 = 1;
    pub const DEFAULT_EXPIRATION_TIME_SECS: u64 = 3600;
    pub const DEFAULT_GAS_TOKEN: &'static str = STC_TOKEN_CODE_STR;

    pub fn new(
        net: ChainNetworkID,
        client: Arc<RpcClient>,
        watch_timeout: Option<Duration>,
        account_client: Box<dyn AccountProvider>,
    ) -> CliStateVM2 {
        let (data_dir, temp_dir) =
            build_dirs_from_net(&net).expect("build dir failed while new CliStateVM2");

        Self {
            net,
            client,
            account_client,
            data_dir,
            temp_dir,
            watch_timeout: watch_timeout.unwrap_or(Self::DEFAULT_WATCH_TIMEOUT),
        }
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    pub fn net(&self) -> &ChainNetworkID {
        &self.net
    }

    pub fn account_client(&self) -> &dyn AccountProvider {
        self.account_client.as_ref()
    }

    pub fn default_account(&self) -> Result<AccountInfo> {
        self.account_client
            .get_default_account()?
            .ok_or_else(|| format_err!("Can not find default account, Please input from account."))
    }

    /// Get account from node managed wallet.
    pub fn get_account(&self, account_address: AccountAddress) -> Result<AccountInfo> {
        self.account_client
            .get_account(account_address)?
            .ok_or_else(|| {
                format_err!("Can not find WalletAccount by address: {}", account_address)
            })
    }

    pub fn get_account_or_default(
        &self,
        account_address: Option<AccountAddress>,
    ) -> Result<AccountInfo> {
        if let Some(account_address) = account_address {
            self.account_client
                .get_account(account_address)?
                .ok_or_else(|| {
                    format_err!("Can not find WalletAccount by address: {}", account_address)
                })
        } else {
            self.default_account()
        }
    }

    pub fn get_resource<R>(&self, address: AccountAddress) -> Result<Bytes>
    where
        R: MoveResource + DeserializeOwned,
    {
        let chain_state_reader = self.client.state_reader2(StateRootOption::Latest)?;
        chain_state_reader.get_resource_type_bytes::<R>(address)
    }

    pub fn get_account_resource(&self, address: AccountAddress) -> Result<AccountResource> {
        self.client
            .state_reader2(StateRootOption::Latest)?
            .get_account_resource(address)
    }

    pub fn association_account(&self) -> Result<Option<AccountInfo>> {
        self.client.account_get2(association_address())
    }

    pub fn watch_txn(&self, txn_hash: HashValue) -> Result<ExecutionOutputView> {
        let block = self
            .client
            .watch_txn(txn_hash, Some(self.watch_timeout))
            .map(Some)
            .unwrap_or_else(|e| {
                eprintln!("Watch txn {:?}  err: {:?}", txn_hash, e);
                None
            });

        let txn_info = {
            const MAX_ATTEMPTS: usize = 2;
            let mut info = None;

            for attempt in 0..MAX_ATTEMPTS {
                if attempt > 0 {
                    std::thread::sleep(Duration::from_secs(5));
                }

                if let Some(result) = self.client.chain_get_transaction_info2(txn_hash)? {
                    info = Some(result);
                    break;
                }
            }

            info.ok_or_else(|| {
                anyhow!(
                    "transaction execute success, but get transaction info return none, block: {}",
                    block
                        .as_ref()
                        .map(|b| b.header.number.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                )
            })?
        };

        let events = self
            .client
            .chain_get_events_by_txn_hash2(txn_hash, Some(GetEventOption { decode: true }))?;

        Ok(ExecutionOutputView::new_with_info(
            txn_hash, txn_info, events,
        ))
    }

    pub fn build_and_execute_transaction(
        &self,
        txn_opts: TransactionOptions,
        payload: TransactionPayload,
    ) -> Result<ExecuteResultView> {
        let (raw_txn, future_transaction) = self.build_transaction(
            txn_opts
                .sender
                .map(|addr| AccountAddress::new(addr.into_bytes())),
            txn_opts.sequence_number,
            txn_opts.gas_unit_price,
            txn_opts.max_gas_amount,
            txn_opts.expiration_time_secs,
            payload,
            txn_opts.gas_token,
        )?;
        if future_transaction {
            //TODO figure out more graceful method to handle future transaction.
            bail!("there is transaction from sender({}) in the txpool, please wait it to been executed or use sequence_number({}) to replace it.",
                raw_txn.sender(), raw_txn.sequence_number() - 1);
        }
        self.execute_transaction(raw_txn, txn_opts.dry_run, txn_opts.blocking)
    }

    fn build_transaction(
        &self,
        sender: Option<AccountAddress>,
        sequence_number: Option<u64>,
        gas_price: Option<u64>,
        max_gas_amount: Option<u64>,
        expiration_time_secs: Option<u64>,
        payload: TransactionPayload,
        gas_token: Option<String>,
    ) -> Result<(RawUserTransaction, bool)> {
        let chain_id = self.net.chain_id();
        let sender = self.get_account_or_default(sender)?;
        let (sequence_number, future_transaction) = match sequence_number {
            Some(sequence_number) => (sequence_number, false),
            None => match self
                .client
                .next_sequence_number2_in_txpool(sender.address)?
            {
                Some(sequence_number) => {
                    info!("get sequence_number {} from txpool", sequence_number);
                    (sequence_number, true)
                }
                None => (
                    self.get_account_resource(*sender.address())?
                        .sequence_number(),
                    false,
                ),
            },
        };
        let node_info = self.client.node_info()?;
        let expiration_timestamp_secs = expiration_time_secs
            .unwrap_or(Self::DEFAULT_EXPIRATION_TIME_SECS)
            + node_info.now_seconds;
        let gas_token_code = gas_token.unwrap_or_else(|| Self::DEFAULT_GAS_TOKEN.to_string());
        Ok((
            RawUserTransaction::new(
                sender.address,
                sequence_number,
                payload,
                max_gas_amount.unwrap_or(Self::DEFAULT_MAX_GAS_AMOUNT),
                gas_price.unwrap_or(Self::DEFAULT_GAS_PRICE),
                expiration_timestamp_secs,
                ChainId::new(chain_id.id()),
                gas_token_code,
            ),
            future_transaction,
        ))
    }

    pub fn dry_run_transaction(&self, txn: DryRunTransaction) -> Result<DryRunOutputView> {
        let state_reader = self.client().state_reader2(StateRootOption::Latest)?;
        playground::dry_run_explain(&state_reader, txn, None)
    }

    pub fn execute_transaction(
        &self,
        raw_txn: RawUserTransaction,
        only_dry_run: bool,
        blocking: bool,
    ) -> Result<ExecuteResultView> {
        let sender = self.get_account(raw_txn.sender())?;
        let public_key = sender.public_key;
        let dry_output = self.dry_run_transaction(DryRunTransaction {
            public_key: public_key.clone(),
            raw_txn: raw_txn.clone(),
        })?;
        let mut raw_txn_view: RawUserTransactionView = raw_txn.clone().try_into()?;
        raw_txn_view.decoded_payload = Some(TransactionPayloadView::from(
            self.decode_txn_payload(raw_txn.payload())?,
        ));

        let mut execute_result = ExecuteResultView::new(raw_txn_view, raw_txn.to_hex(), dry_output);
        if only_dry_run
            || !matches!(
                execute_result.dry_run_output.txn_output.status,
                TransactionStatusView::Executed
            )
        {
            eprintln!(
                "txn dry run result: {:?}",
                execute_result.dry_run_output.txn_output
            );
            return Ok(execute_result);
        }

        let signed_txn = self.account_client.sign_txn(raw_txn, sender.address)?;
        let multisig_public_key = match &public_key {
            AccountPublicKey::Single(_) => {
                let multi_signed_user_txn = MultiSignedUserTransaction::VM2(signed_txn.clone());
                let signed_txn_hex = hex::encode(multi_signed_user_txn.encode()?);
                let txn_hash = self.client.submit_hex_transaction(signed_txn_hex)?;
                eprintln!("txn {} submitted.", txn_hash);
                let execute_output = if blocking {
                    self.watch_txn(txn_hash)?
                } else {
                    ExecutionOutputView::new(txn_hash)
                };
                execute_result.execute_output = Some(execute_output);
                return Ok(execute_result);
            }

            AccountPublicKey::Multi(m) => m.clone(),
        };

        let mut output_dir = current_dir()?;

        let execute_output_view = self.sign_multisig_txn_to_file_or_submit(
            sender.address,
            multisig_public_key,
            None,
            signed_txn,
            &mut output_dir,
            true,
            blocking,
        )?;

        let cur_dir = current_dir()?.to_str().unwrap().to_string();
        if output_dir.to_str().unwrap() != cur_dir {
            // There is signature file, print the file path.
            eprintln!(
                "multisig txn signatures filepath: {}",
                output_dir.to_str().unwrap()
            )
        }

        if let Some(o) = execute_output_view {
            execute_result.execute_output = Some(o)
        };

        Ok(execute_result)
    }

    pub fn decode_txn_payload(
        &self,
        payload: &TransactionPayload,
    ) -> Result<DecodedTransactionPayload> {
        let chain_state_reader = self.client.state_reader2(StateRootOption::Latest)?;
        decode_txn_payload(&chain_state_reader, payload)
    }

    // Sign multisig transaction, if enough signatures collected & submit is true,
    // try to submit txn directly.
    // Otherwise, keep signatures into file.
    pub fn sign_multisig_txn_to_file_or_submit(
        &self,
        sender: AccountAddress,
        multisig_public_key: MultiEd25519PublicKey,
        existing_signatures: Option<MultiEd25519SignatureShard>,
        partial_signed_txn: SignedUserTransaction,
        output_dir: &mut PathBuf,
        submit: bool,
        blocking: bool,
    ) -> Result<Option<ExecutionOutputView>> {
        let my_signatures = if let TransactionAuthenticator::MultiEd25519 { signature, .. } =
            partial_signed_txn.authenticator()
        {
            MultiEd25519SignatureShard::new(signature, *multisig_public_key.threshold())
        } else {
            unreachable!()
        };

        // merge my signatures with existing signatures of other participants.
        let merged_signatures = {
            let mut signatures = vec![];
            if let Some(s) = existing_signatures {
                signatures.push(s);
            }
            signatures.push(my_signatures);
            MultiEd25519SignatureShard::merge(signatures)?
        };
        eprintln!(
            "mutlisig txn(address: {}, threshold: {}): {} signatures collected",
            sender,
            merged_signatures.threshold(),
            merged_signatures.signatures().len()
        );

        let signatures_is_enough = merged_signatures.is_enough();

        if !signatures_is_enough {
            eprintln!(
                "still require {} signatures",
                merged_signatures.threshold() as usize - merged_signatures.signatures().len()
            );
        } else {
            eprintln!("enough signatures collected for the multisig txn, txn can be submitted now");
        }

        // construct the signed txn with merged signatures.
        let signed_txn = {
            let authenticator = TransactionAuthenticator::MultiEd25519 {
                public_key: multisig_public_key,
                signature: merged_signatures.into(),
            };
            SignedUserTransaction::new(partial_signed_txn.into_raw_transaction(), authenticator)
        };

        if submit && signatures_is_enough {
            let execute_output = self.submit_txn(signed_txn, blocking)?;
            return Ok(Some(execute_output));
        }

        // output the txn, send this to other participants to sign, or just submit it.
        let output_file = {
            // use hash's as output file name
            let file_name = signed_txn.crypto_hash().to_hex();
            output_dir.push(file_name);
            output_dir.set_extension("multisig-txn");
            output_dir.clone()
        };
        let mut file = File::create(output_file)?;
        // write txn to file
        bcs_ext::serialize_into(&mut file, &signed_txn)?;
        Ok(None)
    }

    pub fn submit_txn(
        &self,
        signed_txn: SignedUserTransaction,
        blocking: bool,
    ) -> Result<ExecutionOutputView> {
        let mut signed_txn_view: SignedUserTransactionView = signed_txn.clone().try_into()?;
        signed_txn_view.raw_txn.decoded_payload =
            Some(self.decode_txn_payload(signed_txn.payload())?.into());

        eprintln!(
            "Prepare to submit the transaction: \n {}",
            serde_json::to_string_pretty(&signed_txn_view)?
        );
        let txn_hash = signed_txn.id();
        self.client().submit_transaction(signed_txn.into())?;

        eprintln!("txn {:#x} submitted.", txn_hash);

        if blocking {
            self.watch_txn(txn_hash)
        } else {
            Ok(ExecutionOutputView::new(txn_hash))
        }
    }
}
