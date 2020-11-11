use crate::cpu_solver::CpuSolver;
use crate::Solver;
use anyhow::Result;
use libloading::Library;
use starcoin_config::{MinerClientConfig, TimeService};
use std::sync::Arc;

type CreateSolver = extern "C" fn() -> Box<dyn Solver>;

const SOLVER_CREATER: &[u8] = b"create_solver";

pub fn create_solver(
    config: MinerClientConfig,
    time_service: Option<Arc<dyn TimeService>>,
) -> Result<Box<dyn Solver>> {
    match config.plugin_path {
        None => {
            let ts = time_service.expect("time service should exist");
            Ok(Box::new(CpuSolver::new(config, ts)))
        }
        Some(path) => unsafe {
            let lib = Library::new(path).unwrap();
            let call_ref: libloading::Symbol<CreateSolver> = lib.get(SOLVER_CREATER)?;
            Ok(call_ref())
        },
    }
}
