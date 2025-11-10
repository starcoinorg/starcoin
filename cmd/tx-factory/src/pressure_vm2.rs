use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use starcoin_logger::prelude::{error, info};

use crate::mocker::TxnMocker;

pub fn start_vm2_pressure_test(
    mut tx_mocker: TxnMocker,
    round_num: u32,
    account_num: u32,
    batch_size: u32,
    interval: Duration,
    transfer_account_size: usize,
    is_stress: bool,
    stopping_signal: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let accounts = tx_mocker
            .get_or_create_accounts2(account_num, batch_size)
            .expect("create accounts should success");
        while !stopping_signal.load(Ordering::SeqCst) {
            if tx_mocker.get_factory_status() {
                if is_stress {
                    info!("stress account: {}", accounts.len());
                    let success = tx_mocker.stress_test2(
                        accounts.clone(),
                        round_num,
                        interval,
                        transfer_account_size,
                    );
                    if let Err(e) = success {
                        error!("fail to run stress test, err: {:?}", &e);
                        // if txn is rejected, recheck sequence number, and start over
                        if let Err(e) = tx_mocker.recheck_sequence_number2() {
                            error!("fail to start over, err: {:?}", e);
                        }
                    }
                } else {
                    let success = tx_mocker.gen_and_submit_txn2(false);
                    if let Err(e) = success {
                        error!("fail to generate/submit mock txn, err: {:?}", &e);
                        // if txn is rejected, recheck sequence number, and start over
                        if let Err(e) = tx_mocker.recheck_sequence_number2() {
                            error!("fail to start over, err: {:?}", e);
                        }
                    }
                }
            } else {
                info!("txfactory is stop.");
            }

            std::thread::sleep(interval);
        }
    })
}
