use once_cell::sync::Lazy;

pub static RAYON_EXEC_POOL: Lazy<rayon::ThreadPool> = Lazy::new(|| {
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .thread_name(|index| format!("parallel_executor_{}", index))
        .build()
        .expect("failed to build global rayon thread pool for parallel execution")
});
