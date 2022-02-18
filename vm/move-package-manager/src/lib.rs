pub mod compatibility_check_cmd;
pub mod releasement;
use anyhow::Result;
use move_cli::Move;
use move_command_line_common::testing::UPDATE_BASELINE;
use move_compiler::command_line::compiler::construct_pre_compiled_lib_from_compiler;
use move_compiler::diagnostics::report_diagnostics;
use move_compiler::shared::unique_map::UniqueMap;
use move_compiler::{
    cfgir, expansion, hlir, naming, parser, typing, Compiler, FullyCompiledProgram,
};
use move_package::compilation::build_plan::BuildPlan;
use move_package::source_package::layout::SourcePackageLayout;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct TransactionalTestCommand {
    #[structopt(flatten)]
    test_opts: datatest_stable::TestOpts,
    #[structopt(long = "ub")]
    /// update test baseline.
    update_baseline: bool,
}
static PRE_COMPILED_LIB: Lazy<Mutex<Option<FullyCompiledProgram>>> = Lazy::new(|| Mutex::new(None));
pub fn run_transactional_test(move_arg: Move, cmd: TransactionalTestCommand) -> Result<()> {
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
        let compiled = BuildPlan::create(resolved_graph)?
            .compile_with_driver(
                &mut std::io::stdout(),
                |compiler: Compiler, _is_root: bool| {
                    let full_program = match construct_pre_compiled_lib_from_compiler(compiler)? {
                        Ok(full_program) => full_program,
                        Err((file, s)) => report_diagnostics(&file, s),
                    };
                    pre_compiled_lib.files.extend(full_program.files.clone());
                    pre_compiled_lib
                        .parser
                        .lib_definitions
                        .extend(full_program.parser.source_definitions);
                    pre_compiled_lib.expansion.modules =
                        pre_compiled_lib.expansion.modules.union_with(
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
            )?
            .0;
        (pre_compiled_lib, compiled)
    };
    // let (pre_compiled_lib, compiled_pacakge) = {
    //     let compiled_package = move_arg
    //         .build_config
    //         .compile_package(&rerooted_path, &mut std::io::stdout())?;
    //
    //     let pre_compiled_lib = {
    //         let build_root_path = rerooted_path.join(CompiledPackageLayout::Root.path());
    //         let dep_paths = compiled_package
    //             .transitive_dependencies()
    //             .map(|dep_package| {
    //                 build_root_path
    //                     .join(dep_package.compiled_package_info.package_name.to_string())
    //                     .join(CompiledPackageLayout::CompiledModules.path())
    //                     .to_string_lossy()
    //                     .to_string()
    //             })
    //             .collect::<Vec<_>>();
    //         let tmp_interface_dir = tempfile::tempdir()?;
    //         let in_scope_named_addrs = compiled_package
    //             .compiled_package_info
    //             .address_alias_instantiation
    //             .iter()
    //             .map(|(ident, addr)| {
    //                 let parsed_addr =
    //                     NumericalAddress::new(addr.into_bytes(), move_compiler::shared::NumberFormat::Hex);
    //                 (ident.to_string(), parsed_addr)
    //             })
    //             .collect::<BTreeMap<_, _>>();
    //         let pre_compiled_lib = match construct_pre_compiled_lib_from_compiler(
    //             Compiler::new(&[], &dep_paths)
    //                 .set_interface_files_dir(tmp_interface_dir.path().to_string_lossy().to_string())
    //                 .set_flags(Flags::testing())
    //                 .set_named_address_values(in_scope_named_addrs)
    //                 .set_compiled_module_named_address_mapping(
    //                     compiled_package
    //                         .compiled_package_info
    //                         .module_resolution_metadata
    //                         .iter()
    //                         .map(|(k, v)| (k.clone(), v.to_string()))
    //                         .collect(),
    //                 ),
    //         )? {
    //             Ok(full_program) => full_program,
    //             Err((file, s)) => report_diagnostics(&file, s),
    //         };
    //         pre_compiled_lib
    //     };
    //     (pre_compiled_lib, compiled_pacakge)
    // };

    {
        // update the global
        *PRE_COMPILED_LIB.lock().unwrap() = Some(pre_compiled_lib);
    }

    let requirements = datatest_stable::Requirements::new(
        move |path| {
            starcoin_transactional_test_harness::run_test_impl(
                path,
                PRE_COMPILED_LIB.lock().unwrap().as_ref(),
            )
        },
        "transactional-test".to_string(),
        rerooted_path.join("spectests").display().to_string(),
        r".*\.move".to_string(),
    );

    if cmd.update_baseline {
        std::env::set_var(UPDATE_BASELINE, "true");
    }
    datatest_stable::runner_with_opts(&[requirements], cmd.test_opts);
    Ok(())
}
