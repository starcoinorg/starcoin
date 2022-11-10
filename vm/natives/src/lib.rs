// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account;
pub mod hash;
pub mod signature;
pub mod token;
pub mod u256;
// for support evm compat and cross chain.
pub mod ecrecover;

mod helpers;
pub mod util;

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub account: account::GasParameters,
    pub hash: hash::GasParameters,
    pub signature: signature::GasParameters,
    pub token: token::GasParameters,
    pub u256: u256::GasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            account: account::GasParameters {
                create_signer: account::CreateSignerGasParameters { base: 0.into() },
                destroy_signer: account::DestroySignerGasParameters { base: 0.into() },
            },
            hash: hash::GasParameters {
                keccak256: hash::Keccak256HashGasParameters {
                    base: 0.into(),
                    per_byte: 0.into(),
                },
                ripemd160: hash::Ripemd160HashGasParameters {
                    base: 0.into(),
                    per_byte: 0.into(),
                },
            },
            signature: signature::GasParameters {
                ed25519_validate_key: signature::Ed25519ValidateKeyGasParameters {
                    base: 0.into(),
                    per_byte: 0.into(),
                },
                ed25519_verify: signature::Ed25519VerifyGasParameters {
                    base: 0.into(),
                    per_byte: 0.into(),
                },
                ec_recover: ecrecover::EcrecoverGasParameters {
                    base: 0.into(),
                    per_byte: 0.into(),
                },
            },
            token: token::GasParameters {
                token_name_of: token::TokenNameOfGasParameters { base: 0.into() },
            },
            u256: u256::GasParameters {
                add: u256::U256AddGasParameters { base: 0.into() },
                sub: u256::U256SubGasParameters { base: 0.into() },
                mul: u256::U256MulGasParameters { base: 0.into() },
                div: u256::U256DivGasParameters { base: 0.into() },
                rem: u256::U256RemGasParameters { base: 0.into() },
                pow: u256::U256PowGasParameters { base: 0.into() },
                from_bytes: u256::U256FromBytesGasParameters {
                    base: 0.into(),
                    per_byte: 0.into(),
                },
            },
        }
    }
}
