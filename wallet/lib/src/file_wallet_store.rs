// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use scs::SCSCodec;
use starcoin_types::account_address::AccountAddress;
use std::fs::OpenOptions;
use std::path::Path;
use std::{
    fs,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use wallet_api::WalletAccount;
use wallet_api::WalletStore;

pub const DEFAULT_ACCOUNT_FILE_NAME: &str = "account";

/// Save wallet to disk file.
/// Use one dir per account.
pub struct FileWalletStore {
    root_path: PathBuf,
}

impl FileWalletStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if !path.is_dir() {
            fs::create_dir(path).expect("Create wallet dir fail.");
        }

        Self {
            root_path: path.to_owned(),
        }
    }

    fn get_path(&self, address: &AccountAddress, key: &str, is_create: bool) -> Result<PathBuf> {
        let path_str = address.to_string();
        let path = self.root_path.join(path_str);
        if !path.is_dir() && is_create {
            fs::create_dir(path.as_path())?;
        }
        let path = path.join(key);
        Ok(path)
    }

    fn account_dirs(&self) -> Vec<PathBuf> {
        //get account dir from root path
        let root_dir = &self.root_path;
        let mut result = vec![];
        if let Ok(paths) = fs::read_dir(root_dir) {
            for path in paths {
                let tmp_path = path.unwrap().path();
                let tmp_path = tmp_path.join(DEFAULT_ACCOUNT_FILE_NAME);
                result.push(tmp_path);
            }
        }
        result
    }
}

impl WalletStore for FileWalletStore {
    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>> {
        let path = self.get_path(address, DEFAULT_ACCOUNT_FILE_NAME, false)?;
        if path.exists() {
            let file = File::open(&path);
            match file {
                Ok(mut file) => {
                    let mut buffer = vec![];
                    file.read_to_end(&mut buffer)?;
                    let wallet_account = WalletAccount::decode(buffer.as_slice())?;
                    Ok(Some(wallet_account))
                }
                Err(e) => {
                    bail!("open file err: {}", e);
                }
            }
        } else {
            Ok(None)
        }
    }

    fn save_account(&self, account: WalletAccount) -> Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).open(
            self.get_path(&account.address, DEFAULT_ACCOUNT_FILE_NAME, true)
                .unwrap(),
        )?;
        file.write_all(&scs::to_bytes(&account)?)?;
        file.flush()?;
        Ok(())
    }

    fn remove_account(&self, address: &AccountAddress) -> Result<()> {
        let path_str = address.to_string();
        let path = self.root_path.join(path_str);
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    fn get_accounts(&self) -> Result<Vec<WalletAccount>> {
        // get account dir
        let mut result = vec![];
        let account_dirs = self.account_dirs();
        for dir in account_dirs {
            let mut file = File::open(&dir)?;
            let mut buffer = vec![];
            file.read_to_end(&mut buffer)?;
            result.push(WalletAccount::decode(buffer.as_slice()).unwrap());
        }
        Ok(result)
    }

    fn save_to_account(&self, address: &AccountAddress, key: String, value: Vec<u8>) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.get_path(address, &key, true).unwrap())?;
        file.write_all(value.as_slice())?;
        file.flush()?;
        Ok(())
    }

    fn get_from_account(&self, address: &AccountAddress, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_path(address, key, false).unwrap();

        if !path.as_os_str().is_empty() {
            let mut file = File::open(&path)?;
            let mut buffer = vec![];
            file.read_to_end(&mut buffer)?;
            Ok(Option::from(buffer))
        } else {
            Ok(None)
        }
    }
}
