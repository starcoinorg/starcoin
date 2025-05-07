// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{ExecuteResultView, ExecutionOutputView, TransactionOptions};
use anyhow::{bail, format_err, Result};
use bcs_ext::BCSCodec;
use serde::de::DeserializeOwned;
use starcoin_account_api::{AccountInfo, AccountProvider};
use starcoin_config::{ChainNetworkID, DataDirPath};
use starcoin_crypto::{
    multi_ed25519::{multi_shard::MultiEd25519SignatureShard, MultiEd25519PublicKey},
    HashValue, ValidCryptoMaterial,
};
use starcoin_dev::playground;
use starcoin_node::NodeHandle;
use starcoin_rpc_api::multi_dry_run_output_view::MultiDryRunOutputView;
use starcoin_rpc_api::multi_transaction_payload_view::MultiTransactionPayloadView;
use starcoin_rpc_api::types::{RawUserTransactionView, TransactionPayloadView, TransactionStatusView};
use starcoin_rpc_api::{
    chain::GetEventOption, multi_signed_user_transaction_view::MultiSignedUserTransactionView,
};
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_state_api::ChainStateReader;
use starcoin_types::multi_dry_run_transaction::MultiDryRunTransaction;
use starcoin_types::multi_transaction::{MultiAccountAddress, MultiTransactionPayload};
use starcoin_types::{
    account_address::AccountAddress,
    multi_transaction::{MultiRawUserTransaction, MultiSignedUserTransaction},
};
use starcoin_vm2_dev::playground as playground_vm2;
use starcoin_vm2_statedb::{ChainStateDB as ChainStateDB2, ChainStateWriter as ChainStateWriter2};
use starcoin_vm2_storage::Storage as Storage2;
use starcoin_vm2_vm_types::account_config::STC_TOKEN_CODE_STR as STC_TOKEN_CODE_STR_VM2;
use starcoin_vm_types::transaction::authenticator::{AccountPublicKey, TransactionAuthenticator};
use starcoin_vm_types::{
    account_config::{association_address, AccountResource, STC_TOKEN_CODE_STR},
    move_resource::MoveResource,
    state_view::StateReaderExt,
};
use std::{
    convert::TryInto,
    env::current_dir,
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use starcoin_vm2_abi_decoder::{decode_txn_payload as decode_txn_payload_v2};
use starcoin_rpc_api::muti_raw_user_transaction_view::MultiRawUserTransactionView;
use starcoin_vm_types::state_view::StateView as StateViewV2;
use crate::multi_vm::multi_decoded_transaction_payload::MultiDecodedTransactionPayload;
use crate::multi_vm::multi_execute_result_view::MultiExecuteResultView;

static G_HISTORY_FILE_NAME: &str = "history";

pub struct CliStateDualVM {
    net: ChainNetworkID,
    client: Arc<RpcClient>,
    watch_timeout: Duration,
    node_handle: Option<NodeHandle>,
    /// Cli data dir, different with Node data dir.
    data_dir: PathBuf,
    temp_dir: DataDirPath,
    account_client: Box<dyn AccountProvider>,
    mock_db: Arc<dyn StateViewV2>,
}

impl CliStateDualVM {
    pub const DEFAULT_WATCH_TIMEOUT: Duration = Duration::from_secs(300);
    pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 10000000;
    pub const DEFAULT_GAS_PRICE: u64 = 1;
    pub const DEFAULT_EXPIRATION_TIME_SECS: u64 = 3600;
    pub const DEFAULT_GAS_TOKEN: &'static str = STC_TOKEN_CODE_STR;
    pub const DEFAULT_GAS_TOKEN_VM2: &'static str = STC_TOKEN_CODE_STR_VM2;

    pub fn new(
        net: ChainNetworkID,
        client: Arc<RpcClient>,
        watch_timeout: Option<Duration>,
        node_handle: Option<NodeHandle>,
        account_client: Box<dyn AccountProvider>,
    ) -> CliStateDualVM {
        let data_dir = starcoin_config::G_DEFAULT_BASE_DATA_DIR
            .clone()
            .join("cli")
            .join(net.to_string());
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir.as_path())
                .unwrap_or_else(|e| panic!("Create cli data dir {:?} fail, err:{:?}", data_dir, e))
        }
        let temp_dir = data_dir.join("tmp");
        if !temp_dir.exists() {
            std::fs::create_dir_all(temp_dir.as_path())
                .unwrap_or_else(|e| panic!("Create cli temp dir {:?} fail, err:{:?}", temp_dir, e))
        }
        let temp_dir = starcoin_config::temp_dir_in(temp_dir);

        Self {
            net,
            client,
            watch_timeout: watch_timeout.unwrap_or(Self::DEFAULT_WATCH_TIMEOUT),
            node_handle,
            data_dir,
            temp_dir,
            account_client,
        }
    }

    pub fn net(&self) -> &ChainNetworkID {
        &self.net
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    pub fn account_client(&self) -> &dyn AccountProvider {
        self.account_client.as_ref()
    }
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Cli data_dir , ~/.starcoin/cli/$network
    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }

    pub fn history_file(&self) -> PathBuf {
        self.data_dir().join(G_HISTORY_FILE_NAME)
    }

    pub fn node_handle(&self) -> Option<&NodeHandle> {
        self.node_handle.as_ref()
    }

    pub fn default_account(&self) -> Result<AccountInfo> {
        self.account_client
            .get_default_account()?
            .ok_or_else(|| format_err!("Can not find default account, Please input from account."))
    }

    /// Get account from node managed wallet.
    pub fn get_account<T: Into<AccountAddress>>(&self, account_address: T) -> Result<AccountInfo> {
        let address = account_address.into();
        self.account_client
            .get_account(address)?
            .ok_or_else(|| format_err!("Can not find WalletAccount by address: {:?}", address))
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

    pub fn get_resource<R>(&self, address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let chain_state_reader = self.client.state_reader(StateRootOption::Latest)?;
        chain_state_reader.get_resource::<R>(address)
    }

    pub fn get_account_resource(&self, address: AccountAddress) -> Result<Option<AccountResource>> {
        self.get_resource::<AccountResource>(address)
    }

    pub fn association_account(&self) -> Result<Option<AccountInfo>> {
        self.client.account_get(association_address())
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
            if let Some(info) = self.client.chain_get_transaction_info(txn_hash)? {
                info
            } else {
                //sleep and try again.
                std::thread::sleep(Duration::from_secs(5));
                if let Some(info) = self.client.chain_get_transaction_info(txn_hash)? {
                    info
                } else {
                    bail!("transaction execute success, but get transaction info return none, block: {}",
                        block.map(|b|b.header.number.to_string()).unwrap_or_else(||"unknown".to_string()));
                }
            }
        };
        let events = self
            .client
            .chain_get_events_by_txn_hash(txn_hash, Some(GetEventOption { decode: true }))?;

        Ok(ExecutionOutputView::new_with_info(
            txn_hash, txn_info, events,
        ))
    }

    pub fn build_and_execute_transaction(
        &self,
        txn_opts: TransactionOptions,
        payload: MultiTransactionPayload,
    ) -> Result<MultiExecuteResultView> {
        let (raw_txn, future_transaction) = self.build_transaction(
            txn_opts.sender,
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
                raw_txn.sender(),
                raw_txn.sequence_number() - 1
            );
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
        payload: MultiTransactionPayload,
        gas_token: Option<String>,
    ) -> Result<(MultiRawUserTransaction, bool)> {
        let chain_id = self.net().chain_id();
        let sender = self.get_account_or_default(sender)?;
        let (sequence_number, future_transaction) = match sequence_number {
            Some(sequence_number) => (sequence_number, false),
            None => match self.client.next_sequence_number_in_txpool(sender.address)? {
                Some(sequence_number) => {
                    eprintln!("get sequence_number {} from txpool", sequence_number);
                    (sequence_number, true)
                }
                None => self
                    .get_account_resource(*sender.address())?
                    .map(|account| (account.sequence_number(), false))
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find account on chain by address:{}",
                            sender.address()
                        )
                    })?,
            },
        };
        let node_info = self.client.node_info()?;
        let expiration_timestamp_secs = expiration_time_secs
            .unwrap_or(Self::DEFAULT_EXPIRATION_TIME_SECS)
            + node_info.now_seconds;
        // let gas_token_code = gas_token.unwrap_or_else(|| Self::DEFAULT_GAS_TOKEN.to_string());

        let result_txn = match payload {
            MultiTransactionPayload::VM1(payload) => {
                let gas_token_code =
                    gas_token.unwrap_or_else(|| Self::DEFAULT_GAS_TOKEN.to_string());
                MultiRawUserTransaction::VM1(
                    starcoin_vm_types::transaction::RawUserTransaction::new(
                        sender.address,
                        sequence_number,
                        payload,
                        max_gas_amount.unwrap_or(Self::DEFAULT_MAX_GAS_AMOUNT),
                        gas_price.unwrap_or(Self::DEFAULT_GAS_PRICE),
                        expiration_timestamp_secs,
                        chain_id,
                        gas_token_code,
                    ),
                )
            }
            MultiTransactionPayload::VM2(payload) => {
                let gas_token_code =
                    gas_token.unwrap_or_else(|| Self::DEFAULT_GAS_TOKEN_VM2.to_string());
                MultiRawUserTransaction::VM2(
                    starcoin_vm2_vm_types::transaction::RawUserTransaction::new(
                        starcoin_vm2_vm_types::account_address::AccountAddress::new(
                            sender.address.into_bytes(),
                        ),
                        sequence_number,
                        payload,
                        max_gas_amount.unwrap_or(Self::DEFAULT_MAX_GAS_AMOUNT),
                        gas_price.unwrap_or(Self::DEFAULT_GAS_PRICE),
                        expiration_timestamp_secs,
                        starcoin_vm2_vm_types::genesis_config::ChainId::new(chain_id.id()),
                        gas_token_code,
                    ),
                )
            }
        };
        Ok((result_txn, future_transaction))
    }

    pub fn dry_run_transaction(
        &self,
        txn: MultiDryRunTransaction,
    ) -> Result<MultiDryRunOutputView> {
        Ok(match txn {
            MultiDryRunTransaction::VM1(txn) => {
                let state_reader = self.client().state_reader(StateRootOption::Latest)?;
                MultiDryRunOutputView::VM1(playground::dry_run_explain(&state_reader, txn, None)?)
            }
            MultiDryRunTransaction::VM2(txn) => {
                let state_reader = self.client().state_reader(StateRootOption::Latest)?;
                let (_, _state_root_vm2) = state_reader.get_multi_vm_state_roots()?;

                // TODO(BobOng): [dual-vm] To Get Storage for construction StateView
                // let state_view = ChainStateDB2::new(store, state_root_vm2);
                let state_view = ChainStateDB2::mock();
                MultiDryRunOutputView::VM2(playground_vm2::dry_run_explain(&state_view, txn, None)?)
            }
        })
    }

    pub fn execute_transaction(
        &self,
        raw_txn: MultiRawUserTransaction,
        only_dry_run: bool,
        blocking: bool,
    ) -> Result<MultiExecuteResultView> {
        let sender = self.get_account(raw_txn.sender())?;
        let public_key = sender.public_key;

        let multi_txn =
            self.build_multi_dry_run_transaction(raw_txn.clone(), public_key.clone())?;
        let dry_output = self.dry_run_transaction(multi_txn)?;

        let mut raw_txn_view: MultiRawUserTransactionView = raw_txn.clone().try_into()?;

        // TODO(BobOng): [dual-vm] decode payload
        // raw_txn_view.decoded_payload = Some(self.decode_txn_payload(&raw_txn.payload())?);

        let mut execute_result = MultiExecuteResultView::new(
            raw_txn_view,
            raw_txn.to_hex(),
            dry_output.try_into().unwrap(),
        );
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

        // TODO(BobOng):[dual-vm] How to sign txn using vm2?
        let signed_txn = self
            .account_client
            .sign_txn(raw_txn.into(), sender.address)?;

        let multisig_public_key = match &public_key {
            AccountPublicKey::Single(_) => {
                let signed_txn_hex = hex::encode(signed_txn.encode()?);
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
        payload: &MultiTransactionPayload,
    ) -> Result<MultiDecodedTransactionPayload> {
        let chain_state_reader = self.client.state_reader(StateRootOption::Latest)?;
        Ok(match payload {
            MultiTransactionPayload::VM1(payload) => {
                MultiDecodedTransactionPayload::VM1(decode_txn_payload(&chain_state_reader, payload)?),
            }
            MultiTransactionPayload::VM2(payload) => {
                MultiDecodedTransactionPayload::VM2(decode_txn_payload_v2(self.get_vm2_state_view(), payload)?)
            }
        })
    }

    pub fn into_inner(self) -> (ChainNetworkID, Arc<RpcClient>, Option<NodeHandle>) {
        (self.net, self.client, self.node_handle)
    }

    // Sign multisig transaction, if enough signatures collected & submit is true,
    // try to submit txn directly.
    // Otherwise, keep signatures into file.
    pub fn sign_multisig_txn_to_file_or_submit(
        &self,
        sender: MultiAccountAddress,
        multisig_public_key: MultiEd25519PublicKey,
        existing_signatures: Option<MultiEd25519SignatureShard>,
        partial_signed_txn: MultiSignedUserTransaction,
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
        signed_txn: MultiSignedUserTransaction,
        blocking: bool,
    ) -> Result<ExecutionOutputView> {
        let mut signed_txn_view: MultiSignedUserTransactionView = signed_txn.clone().try_into()?;
        signed_txn_view.raw_txn.decoded_payload =
            Some(self.decode_txn_payload(&signed_txn.payload())?.into());

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

    fn build_multi_dry_run_transaction(
        &self,
        txn: MultiRawUserTransaction,
        sender_pub_key: AccountPublicKey,
    ) -> Result<MultiDryRunTransaction> {
        let multi_txn = match txn {
            MultiRawUserTransaction::VM1(raw_txn) => {
                MultiDryRunTransaction::VM1(starcoin_vm_types::transaction::DryRunTransaction {
                    raw_txn,
                    public_key: sender_pub_key.clone(),
                })
            }
            MultiRawUserTransaction::VM2(raw_txn) => {
                MultiDryRunTransaction::VM2(starcoin_vm2_vm_types::transaction::DryRunTransaction {
                    raw_txn: starcoin_vm2_vm_types::transaction::RawUserTransaction::new(
                        starcoin_vm2_vm_types::account_address::AccountAddress::new(
                            raw_txn.sender().into_bytes(),
                        ),
                        raw_txn.sequence_number(),
                        raw_txn.payload().clone(),
                        raw_txn.max_gas_amount(),
                        raw_txn.gas_unit_price(),
                        raw_txn.expiration_timestamp_secs(),
                        starcoin_vm2_vm_types::genesis_config::ChainId::new(
                            raw_txn.chain_id().id(),
                        ),
                        raw_txn.gas_token_code(),
                    ),
                    public_key: starcoin_vm2_vm_types::transaction::authenticator::AccountPublicKey::try_from(sender_pub_key.to_bytes().as_slice())?,
                })
            }
        };
        Ok(multi_txn)
    }

    fn get_vm2_state_view(&self) -> &dyn StateViewV2 {
        // TODO(BobOng): [dual-vm] To Get Storage for construction StateView
        self.mock_db.as_ref()
    }
}
