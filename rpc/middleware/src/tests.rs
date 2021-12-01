use super::*;
use futures::executor::block_on;
use jsonrpc_core::{MetaIoHandler, Params, Value};
use rand::Rng;
use starcoin_metrics::{get_all_metrics, Registry};
use std::time::Duration;

#[stest::test]
fn test_middleware() {
    let registry = Registry::new();
    let metrics = RpcMetrics::register(&registry).unwrap();
    let mut io_handler = MetaIoHandler::with_middleware(MetricMiddleware::new(Some(metrics)));
    io_handler.add_method("status", |_params: Params| async {
        let mut rng = rand::thread_rng();
        let sleep_time = rng.gen_range(1..50);
        std::thread::sleep(Duration::from_millis(sleep_time));
        Ok(Value::Bool(true))
    });
    let request_tmpl_prefix = r#"{"jsonrpc":"2.0","method":"status","params":[],"id":"#;
    let request_tmpl_suffix = "}";
    //let response = r#"{"jsonrpc":"2.0","result":true,"id":0}"#;
    let count = 10;
    let mut futs = vec![];
    for i in 0..count {
        let request = format!("{}{}{}", request_tmpl_prefix, i, request_tmpl_suffix);
        let fut = io_handler.handle_request(request.as_str(), Metadata::default());
        futs.push(fut);
    }
    for fut in futs {
        assert!(block_on(fut).is_some());
    }
    info!("metrics: {:?}", get_all_metrics(&registry));
}
