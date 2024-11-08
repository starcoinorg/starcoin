use std::{sync::Arc, time::Duration};

use starcoin_sync::parallel::worker_scheduler::WorkerScheduler;
use tokio::{task, time};

struct Worker {
  worker_scheduler: Arc<WorkerScheduler>,
}

impl Worker {
  pub fn new(worker_scheduler: Arc<WorkerScheduler>) -> Self {
      Worker {
          worker_scheduler,
      }
  }

  pub async fn start(&self) {
      self.worker_scheduler.worker_start();
      loop {
          if self.worker_scheduler.check_if_stop().await {
              break;
          }
          println!("Worker is working, worker_count = {:?}", self.worker_scheduler.check_worker_count().await);
          time::sleep(Duration::from_secs(1)).await;
      }
      self.worker_scheduler.worker_exits();
  }
  
}

async fn work_cycle(worker_scheduler: Arc<WorkerScheduler>) {
  worker_scheduler.tell_worker_to_start().await;
  for _i in 0..10 {
      let worker = Worker::new(worker_scheduler.clone());
      task::spawn(async move {
          worker.start().await;
      });
  }

  println!("Start worker, now sleep for 5 seconds");
  time::sleep(Duration::from_secs(5)).await;

  println!("now stop worker");
  worker_scheduler.tell_worker_to_stop().await;

  worker_scheduler.wait_for_worker().await;
}

#[stest::test(timeout = 120)]
async fn test_sync_parallel_scheduler() {
  let worker_scheduler = Arc::new(WorkerScheduler::new());
  for i in 0..10 {
      println!("********************* work_cycle {} start", i);
      worker_scheduler.tell_worker_to_stop().await;
      worker_scheduler.wait_for_worker().await;
      work_cycle(worker_scheduler.clone()).await;
      println!("********************* work_cycle {} end", i);
  }
}