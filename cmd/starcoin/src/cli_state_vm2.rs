// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    view::TransactionOptions,
    view_vm2::{ExecuteResultView, ExecutionOutputView},
};
use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use starcoin_account_api::{AccountInfo, AccountProvider};// TODO(BobOng):[dual-vm] to change account info vm2

use starcoin_rpc_client::RpcClient;
use starcoin_vm2_abi_decoder::DecodedTransactionPayload;
use starcoin_vm2_crypto::{
    hash::PlainCryptoHash,
    multi_ed25519::{multi_shard::MultiEd25519SignatureShard, MultiEd25519PublicKey},
    HashValue,
};
use starcoin_vm2_types::view::{DryRunOutputView, SignedUserTransactionView};
use starcoin_vm2_vm_types::{
    account_address::AccountAddress,
    account_config::{AccountResource, STC_TOKEN_CODE_STR},
    move_resource::MoveResource,
    transaction::authenticator::TransactionAuthenticator,
    transaction::{
        DryRunTransaction, RawUserTransaction, SignedUserTransaction, TransactionPayload,
    },
};
use std::{fs::File, path::PathBuf, sync::Arc, time::Duration};
use starcoin_config::DataDirPath;

/// A reduced version of clistate, retaining only the necessary execution action functions
#[allow(dead_code)]
pub struct CliStateVM2 {
    client: Arc<RpcClient>, // TODO(BobOng):[dual-vm] to change rpc vm2
    // account_provider: Box<dyn AccountProvider>,  // TODO(BobOng):[dual-vm] to change vm2 provider
    watch_timeout: Duration,
    data_dir: PathBuf,
    temp_dir: DataDirPath,
}

impl CliStateVM2 {
    pub const DEFAULT_WATCH_TIMEOUT: Duration = Duration::from_secs(300);
    pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 10000000;
    pub const DEFAULT_GAS_PRICE: u64 = 1;
    pub const DEFAULT_EXPIRATION_TIME_SECS: u64 = 3600;
    pub const DEFAULT_GAS_TOKEN: &'static str = STC_TOKEN_CODE_STR;

    pub fn new(
        client: Arc<RpcClient>,
        watch_timeout: Option<Duration>
        // account_provider: Box<dyn AccountProvider>,  // TODO(BobOng):[dual-vm] to change vm2 provider
    ) -> CliStateVM2 {
        // TODO(BobOng):[dual-vm] to change rpc vm2
        Self {
            client,
            // account_provider, // TODO(BobOng):[dual-vm] to change vm2 provider
            data_dir: PathBuf::new(), // TODO(BobOng): [dual-vm] to intialize dir for vm2
            temp_dir: DataDirPath::default(), // TODO(BobOng): [dual-vm] to intialize dir for vm2
            watch_timeout: watch_timeout.unwrap_or(Self::DEFAULT_WATCH_TIMEOUT),
        }
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }


    pub fn default_account(&self) -> Result<AccountInfo> {
        // TODO(BobOng): [dual-vm] get account info from vm2 provider
        unimplemented!()
    }

    /// Get account from node managed wallet.
    pub fn get_account(&self, _account_address: AccountAddress) -> Result<AccountInfo> {
        // TODO(BobOng): [dual-vm] get account info from vm2 provider
        unimplemented!()
    }

    pub fn get_account_or_default(
        &self,
        _account_address: Option<AccountAddress>,
    ) -> Result<AccountInfo> {
        // TODO(BobOng): [dual-vm] get account from rpc vm2
        unimplemented!()
    }

    pub fn get_resource<R>(&self, _address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        // TODO(BobOng): [dual-vm] get resource from chain state reader vm2
        unimplemented!()
    }

    pub fn get_account_resource(&self, address: AccountAddress) -> Result<Option<AccountResource>> {
        self.get_resource::<AccountResource>(address)
    }

    pub fn association_account(&self) -> Result<Option<AccountInfo>> {
        // TODO(BobOng): [dual-vm] get account info for vm2
        unimplemented!()
    }

    pub fn watch_txn(&self, _txn_hash: HashValue) -> Result<ExecutionOutputView> {
        // TODO(BobOng): [dual-vm] to implement
        unimplemented!()
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
        _sender: Option<AccountAddress>,
        _sequence_number: Option<u64>,
        _gas_price: Option<u64>,
        _max_gas_amount: Option<u64>,
        _expiration_time_secs: Option<u64>,
        _payload: TransactionPayload,
        _gas_token: Option<String>,
    ) -> Result<(RawUserTransaction, bool)> {
        // TODO(BobOng): [dual-vm] to build transaction for vm2
        unimplemented!()
    }

    pub fn dry_run_transaction(&self, _txn: DryRunTransaction) -> Result<DryRunOutputView> {
        // TODO(BobOng): [dual-vm] to dry run transaction for vm2
        unimplemented!()
    }

    pub fn execute_transaction(
        &self,
        _raw_txn: RawUserTransaction,
        _only_dry_run: bool,
        _blocking: bool,
    ) -> Result<ExecuteResultView> {
        // TODO(BobOng): [dual-vm] to execute transaction for vm2
        unimplemented!()
    }

    pub fn decode_txn_payload(
        &self,
        _payload: &TransactionPayload,
    ) -> Result<DecodedTransactionPayload> {
        // TODO(BobOng): [dual-vm] to decode transaction payload transaction for vm2
        unimplemented!()
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
