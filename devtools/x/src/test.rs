// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::cargo::{CargoArgs, CargoCommand};
use crate::project_root;
use anyhow::{Error, Result};
use libra_x::{
    // cargo::{CargoArgs, CargoCommand},
    context::XContext,
    utils,
    // utils::project_root,
};
use log::info;
use std::{
    ffi::OsString,
    fs::create_dir_all,
    path::PathBuf,
    process::{Command, Stdio},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short, number_of_values = 1)]
    /// Run test on the provided packages
    package: Vec<String>,
    #[structopt(long, short)]
    /// Skip running expensive libra testsuite integration tests
    unit: bool,
    #[structopt(long)]
    /// Test only this package's library unit tests, skipping doctests
    lib: bool,
    #[structopt(long)]
    /// Do not fast fail the run if tests (or test executables) fail
    no_fail_fast: bool,
    #[structopt(long)]
    /// Do not run tests, only compile the test executables
    no_run: bool,
    #[structopt(long, short)]
    /// Number of parallel jobs, defaults to # of CPUs
    jobs: Option<u16>,
    #[structopt(long, parse(from_os_str))]
    /// Directory to output HTML coverage report (using grcov)
    html_cov_dir: Option<PathBuf>,
    #[structopt(long, parse(from_os_str))]
    /// Directory to output lcov coverage html (using grcov -> lcov.info -> html using genhtml).
    /// Only useful if you want the lcov.info file produced in the path.  Requires that lcov be installed and on PATH.
    html_lcov_dir: Option<PathBuf>,
    #[structopt(name = "TESTNAME", parse(from_os_str))]
    testname: Option<OsString>,
    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(mut args: Args, xctx: XContext) -> Result<()> {
    args.args.extend(args.testname.clone());

    let config = xctx.config();

    let generate_coverage = args.html_cov_dir.is_some() || args.html_lcov_dir.is_some();

    let env_vars: &[(&str, &str)] = if generate_coverage {
        info!("Running \"cargo clean\" before collecting coverage");
        let mut clean_cmd = Command::new("cargo");
        clean_cmd.arg("clean");
        clean_cmd.output()?;
        &[
            // A way to use -Z (unstable) flags with the stable compiler. See below.
            ("RUSTC_BOOTSTRAP", "1"),
            // Recommend setting for grcov, avoids using the cargo cache.
            ("CARGO_INCREMENTAL", "0"),
            // language/ir-testsuite's tests will stack overflow without this setting.
            ("RUST_MIN_STACK", "8388608"),
            // Recommend flags for use with grcov, with these flags removed: -Copt-level=0, -Clink-dead-code.
            // for more info see:  https://github.com/mozilla/grcov#example-how-to-generate-gcda-fiels-for-a-rust-project
            (
                "RUSTFLAGS",
                "-Zprofile -Ccodegen-units=1 -Coverflow-checks=off",
            ),
        ]
    } else {
        &[]
    };

    let mut direct_args = Vec::new();
    if args.no_run {
        direct_args.push(OsString::from("--no-run"));
    };
    if args.no_fail_fast {
        direct_args.push(OsString::from("--no-fail-fast"));
    };
    if args.lib {
        direct_args.push(OsString::from("--lib"));
    };

    if let Some(jobs) = args.jobs {
        direct_args.push(OsString::from("--jobs"));
        direct_args.push(OsString::from(jobs.to_string()));
    };

    let cmd = CargoCommand::Test {
        cargo_config: xctx.config().cargo_config(),
        direct_args: direct_args.as_slice(),
        args: &args.args,
        env: &env_vars,
    };

    let cmd_result = if args.unit {
        cmd.run_with_exclusions(
            config.system_tests().iter().map(|(p, _)| p),
            &CargoArgs::default(),
        )
    } else if !args.package.is_empty() {
        cmd.run_on_packages(args.package.iter(), &CargoArgs::default())
    } else if utils::project_is_root(&xctx)? {
        // TODO Maybe only run a subest of tests if we're not inside
        // a package but not at the project root (e.g. language)
        cmd.run_on_all_packages(&CargoArgs::default())
    } else {
        cmd.run_on_local_package(&CargoArgs::default())
    };

    if !args.no_fail_fast && cmd_result.is_err() {
        return cmd_result;
    }

    if let Some(html_cov_dir) = &args.html_cov_dir {
        create_dir_all(&html_cov_dir)?;
        let html_cov_path = &html_cov_dir.canonicalize()?;
        info!("created {}", &html_cov_path.to_string_lossy());
        exec_grcov(&html_cov_path)?;
    }
    if let Some(html_lcov_dir) = &args.html_lcov_dir {
        create_dir_all(&html_lcov_dir)?;
        let html_lcov_path = &html_lcov_dir.canonicalize()?;
        info!("created {}", &html_lcov_path.to_string_lossy());
        exec_lcov(&html_lcov_path)?;
        exec_lcov_genhtml(&html_lcov_path)?;
    }
    cmd_result
}

fn exec_lcov_genhtml(html_lcov_path: &PathBuf) -> Result<()> {
    let mut genhtml = Command::new("genhtml");
    let mut lcov_file_path = PathBuf::new();
    lcov_file_path.push(html_lcov_path);
    lcov_file_path.push("lcov.info");
    genhtml
        .current_dir(project_root())
        .arg("-o")
        .arg(html_lcov_path)
        .arg("--show-details")
        .arg("--highlight")
        .arg("--ignore-errors")
        .arg("source")
        .arg("--legend")
        //TODO: Paths seem to be a thing
        .arg(lcov_file_path);
    info!("Build grcov lcov.info file");
    info!("{:?}", genhtml);
    genhtml.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    if let Some(err) = genhtml.output().err() {
        Err(Error::new(err).context("Failed to generate html output from lcov.info"))
    } else {
        Ok(())
    }
}

fn exec_lcov(html_lcov_path: &PathBuf) -> Result<()> {
    let debug_dir = project_root().join("target/debug/");
    let mut lcov_file_path = PathBuf::new();
    lcov_file_path.push(html_lcov_path);
    lcov_file_path.push("lcov.info");
    let mut lcov_file = Command::new("grcov");
    lcov_file
        .current_dir(project_root())
        //output file from coverage: gcda files
        .arg(debug_dir.as_os_str())
        //source code location
        .arg("-s")
        .arg(project_root().as_os_str())
        //html output
        .arg("-t")
        .arg("lcov")
        .arg("--llvm")
        .arg("--branch")
        .arg("--ignore")
        .arg("/*")
        .arg("--ignore")
        .arg("x/*")
        .arg("--ignore")
        .arg("testsuite/*")
        .arg("--ignore-not-existing")
        .arg("-o")
        //TODO: Paths seem to be a thing
        .arg(lcov_file_path);
    info!("Converting lcov.info file to html");
    info!("{:?}", lcov_file);
    lcov_file.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    if let Some(err) = lcov_file.output().err() {
        Err(Error::new(err).context("Failed to generate lcov.info with grcov"))
    } else {
        Ok(())
    }
}

fn exec_grcov(html_cov_path: &PathBuf) -> Result<()> {
    let debug_dir = project_root().join("target/debug/");
    let mut grcov_html = Command::new("grcov");
    grcov_html
        .current_dir(project_root())
        //output file from coverage: gcda files
        .arg(debug_dir.as_os_str())
        //source code location
        .arg("-s")
        .arg(project_root().as_os_str())
        //html output
        .arg("-t")
        .arg("html")
        .arg("--llvm")
        .arg("--branch")
        .arg("--ignore")
        .arg("/*")
        .arg("--ignore")
        .arg("x/*")
        .arg("--ignore")
        .arg("benchmarks/*")
        .arg("--ignore")
        .arg("vm/transaction-builder-generator")
        .arg("--ignore")
        .arg("testsuite/*")
        .arg("--ignore-not-existing")
        .arg("-o")
        .arg(html_cov_path);
    info!("Build grcov Html Coverage Report");
    info!("{:?}", grcov_html);
    grcov_html.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    if let Some(err) = grcov_html.output().err() {
        Err(Error::new(err).context("Failed to generate html output with grcov"))
    } else {
        Ok(())
    }
}
