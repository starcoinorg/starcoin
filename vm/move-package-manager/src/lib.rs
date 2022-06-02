// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::{Args, Parser};
use move_cli::Move;
use move_command_line_common::testing::UPDATE_BASELINE;
use move_compiler::command_line::compiler::construct_pre_compiled_lib_from_compiler;
use move_compiler::diagnostics::report_diagnostics;
use move_compiler::shared::unique_map::UniqueMap;
use move_compiler::shared::NamedAddressMaps;
use move_compiler::{
    cfgir, expansion, hlir, naming, parser, typing, Compiler, FullyCompiledProgram,
};
use move_package::compilation::build_plan::BuildPlan;
use move_package::source_package::layout::SourcePackageLayout;
use once_cell::sync::Lazy;
use std::fmt::Display;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

pub mod compatibility_check_cmd;
pub mod releasement;

// use `integration-tests` rather than `tests`, for avoid conflict with `mpm package test`
pub const INTEGRATION_TESTS_DIR: &str = "integration-tests";

#[derive(Debug, Args)]
pub struct TestOpts {
    /// The FILTER string is tested against the name of all tests, and only those tests whose names
    /// contain the filter are run.
    filter: Option<String>,

    #[clap(long = "exact")]
    /// Exactly match filters rather than by substring
    filter_exact: bool,

    #[clap(long, env = "RUST_TEST_THREADS", default_value = "32")]
    /// Number of threads used for running tests in parallel
    test_threads: NonZeroUsize,

    #[clap(short = 'q', long)]
    /// Output minimal information
    quiet: bool,

    #[clap(long)]
    /// List all tests
    list: bool,

    #[clap(long)]
    /// Configure formatting of output:
    ///   pretty = Print verbose output;
    ///   terse = Display one character per test;
    ///   (json is unsupported, exists for compatibility with the default test harness)
    #[clap(possible_values = Format::variants(), default_value_t, ignore_case = true)]
    format: Format,
}

#[derive(Debug, Eq, PartialEq)]
enum Format {
    Pretty,
    Terse,
    Json,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Pretty => write!(f, "pretty"),
            Format::Terse => write!(f, "terse"),
            Format::Json => write!(f, "json"),
        }
    }
}

impl Format {
    fn variants() -> Vec<&'static str> {
        vec!["pretty", "terse"]
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Format, std::string::String> {
        match s {
            "pretty" => Ok(Format::Pretty),
            "terse" => Ok(Format::Terse),
            "json" => Ok(Format::Json),
            _ => Err(format!("Unsupported format: {}", s)),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Pretty
    }
}

#[derive(Args, Debug)]
pub struct IntegrationTestCommand {
    #[clap(flatten)]
    test_opts: TestOpts,
    #[clap(long = "ub")]
    /// update test baseline.
    update_baseline: bool,
}
static G_PRE_COMPILED_LIB: Lazy<Mutex<Option<FullyCompiledProgram>>> =
    Lazy::new(|| Mutex::new(None));
pub fn run_integration_test(move_arg: Move, cmd: IntegrationTestCommand) -> Result<()> {
    let rerooted_path = {
        let path = &move_arg.package_path;
        // Always root ourselves to the package root, and then compile relative to that.
        let rooted_path = SourcePackageLayout::try_find_root(&path.canonicalize()?)?;
        std::env::set_current_dir(&rooted_path).unwrap();
        PathBuf::from(".")
    };
    let (pre_compiled_lib, _compiled_package) = {
        // force move to rebuild all packages, so that we can use compile_driver to generate the full compiled program.
        let mut build_config = move_arg.build_config;
        build_config.force_recompilation = true;
        let resolved_graph = build_config.resolution_graph_for_package(&rerooted_path)?;
        let mut pre_compiled_lib = FullyCompiledProgram {
            files: Default::default(),
            parser: parser::ast::Program {
                named_address_maps: NamedAddressMaps::new(),
                source_definitions: vec![],
                lib_definitions: vec![],
            },
            expansion: expansion::ast::Program {
                modules: UniqueMap::new(),
                scripts: Default::default(),
            },
            naming: naming::ast::Program {
                modules: UniqueMap::new(),
                scripts: Default::default(),
            },
            typing: typing::ast::Program {
                modules: UniqueMap::new(),
                scripts: Default::default(),
            },
            hlir: hlir::ast::Program {
                modules: UniqueMap::new(),
                scripts: Default::default(),
            },
            cfgir: cfgir::ast::Program {
                modules: UniqueMap::new(),
                scripts: Default::default(),
            },
            compiled: vec![],
        };
        let compiled = BuildPlan::create(resolved_graph)?.compile_with_driver(
            &mut std::io::stdout(),
            |compiler: Compiler| {
                let full_program = match construct_pre_compiled_lib_from_compiler(compiler)? {
                    Ok(full_program) => full_program,
                    Err((file, s)) => report_diagnostics(&file, s),
                };
                pre_compiled_lib.files.extend(full_program.files.clone());
                pre_compiled_lib
                    .parser
                    .lib_definitions
                    .extend(full_program.parser.source_definitions);
                pre_compiled_lib.expansion.modules = pre_compiled_lib.expansion.modules.union_with(
                    &full_program.expansion.modules.filter_map(|_k, v| {
                        if v.is_source_module {
                            Some(v)
                        } else {
                            None
                        }
                    }),
                    |_k, v1, _v2| v1.clone(),
                );
                pre_compiled_lib.naming.modules = pre_compiled_lib.naming.modules.union_with(
                    &full_program.naming.modules.filter_map(|_k, v| {
                        if v.is_source_module {
                            Some(v)
                        } else {
                            None
                        }
                    }),
                    |_k, v1, _v2| v1.clone(),
                );
                pre_compiled_lib.typing.modules = pre_compiled_lib.typing.modules.union_with(
                    &full_program.typing.modules.filter_map(|_k, v| {
                        if v.is_source_module {
                            Some(v)
                        } else {
                            None
                        }
                    }),
                    |_k, v1, _v2| v1.clone(),
                );
                pre_compiled_lib.hlir.modules = pre_compiled_lib.hlir.modules.union_with(
                    &full_program.hlir.modules.filter_map(|_k, v| {
                        if v.is_source_module {
                            Some(v)
                        } else {
                            None
                        }
                    }),
                    |_k, v1, _v2| v1.clone(),
                );
                pre_compiled_lib.cfgir.modules = pre_compiled_lib.cfgir.modules.union_with(
                    &full_program.cfgir.modules.filter_map(|_k, v| {
                        if v.is_source_module {
                            Some(v)
                        } else {
                            None
                        }
                    }),
                    |_k, v1, _v2| v1.clone(),
                );
                pre_compiled_lib
                    .compiled
                    .extend(full_program.compiled.clone());

                Ok((full_program.files, full_program.compiled))
            },
        )?;
        (pre_compiled_lib, compiled)
    };

    {
        // update the global
        *G_PRE_COMPILED_LIB.lock().unwrap() = Some(pre_compiled_lib);
    }

    let spectests_dir = rerooted_path.join("spectests");
    // for compatibility with old version mpm, check the spectests first.
    let tests_dir = if spectests_dir.exists() && spectests_dir.is_dir() {
        eprintln!(
            r#"
            Note: The new version of mpm changes the `spectest` to `integration-test`, and use the `integration-tests` dir.
            You can just move the `spectests` to `integration-tests`.
            "#
        );
        spectests_dir
    } else {
        rerooted_path.join(INTEGRATION_TESTS_DIR)
    };

    if !tests_dir.exists() || !tests_dir.is_dir() {
        eprintln!("No integration tests file in the dir `integration-tests`.");
        return Ok(());
    }

    let requirements = datatest_stable::Requirements::new(
        move |path| {
            starcoin_transactional_test_harness::run_test_impl(
                path,
                G_PRE_COMPILED_LIB.lock().unwrap().as_ref(),
            )
        },
        "integration-test".to_string(),
        tests_dir.display().to_string(),
        r".*\.move".to_string(),
    );

    if cmd.update_baseline {
        std::env::set_var(UPDATE_BASELINE, "true");
    }
    let mut test_args = vec![
        "test_runner".to_string(),
        "--format".to_string(),
        cmd.test_opts.format.to_string(),
        "--test-threads".to_string(),
        cmd.test_opts.test_threads.to_string(),
    ];
    if cmd.test_opts.list {
        test_args.push("--list".to_string());
    }
    if cmd.test_opts.quiet {
        test_args.push("--quiet".to_string());
    }
    if cmd.test_opts.filter_exact {
        test_args.push("--exact".to_string());
    }

    if let Some(filter) = cmd.test_opts.filter {
        test_args.push("--".to_string());
        test_args.push(filter);
    }

    let test_opts = datatest_stable::TestOpts::try_parse_from(test_args.as_slice())?;
    datatest_stable::runner_with_opts(&[requirements], test_opts);
    Ok(())
}
