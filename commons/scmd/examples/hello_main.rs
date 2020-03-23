// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use scmd::CmdContext;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "alpha")]
struct AlphaOpts {
    #[structopt(short = "n")]
    name: String,
}

fn main() -> Result<()> {
    let context = CmdContext::new("hello_app");
    context
        .with_command::<AlphaOpts>(Box::new(|args| -> Result<()> {
            println!("hello alpha {:?}", args.name);
            Ok(())
        }))
        .execute()?;
    Ok(())
}
