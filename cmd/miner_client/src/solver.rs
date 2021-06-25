use crate::cpu_solver::CpuSolver;
use anyhow::Result;
use starcoin_config::{MinerClientConfig, TimeService};
use starcoin_miner_client_api::Solver;
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
            //Since this issue https://github.com/nagisa/rust_libloading/issues/41
            #[cfg(target_os = "linux")]
            let lib = libloading::os::unix::Library::open(Some(path), 0x2 | 0x1000)?;
            #[cfg(not(target_os = "linux"))]
            let lib = libloading::Library::new(path)?;
            let call_ref = lib.get::<CreateSolver>(SOLVER_CREATER)?;

            Ok(call_ref())
        },
    }
}
