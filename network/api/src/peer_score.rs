use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct ScoreCounter {
    score: Arc<AtomicU64>,
    count: Arc<AtomicU64>,
}

impl ScoreCounter {
    pub fn inc_by(&self, score: i64) {
        self.score.fetch_add(score as u64, Ordering::SeqCst);
        self.count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn score(&self) -> u64 {
        self.score.load(Ordering::SeqCst)
    }

    pub fn avg(&self) -> u64 {
        self.score() / self.count.load(Ordering::SeqCst)
    }
}

impl Default for ScoreCounter {
    fn default() -> Self {
        Self {
            score: Arc::new(AtomicU64::new(1)),
            count: Arc::new(AtomicU64::new(0)),
        }
    }
}

pub trait Score<Entry>: Sync + Send {
    fn execute(&self, entry: Entry) -> i64;
}

#[derive(Clone)]
pub struct InverseScore {
    k: i64,
}

impl InverseScore {
    pub fn new(x: u32, y: u32) -> Self {
        assert!(y < 100);
        Self { k: (x * y) as i64 }
    }
}

impl Score<u32> for InverseScore {
    fn execute(&self, time: u32) -> i64 {
        assert!(time > 0);
        self.k / (time as i64)
    }
}

pub enum HandleState {
    Future,
    Succ,
    Fail,
}

pub struct BlockBroadcastEntry {
    new: bool,
    state: HandleState,
}

impl BlockBroadcastEntry {
    pub fn new(new: bool, state: HandleState) -> Self {
        Self { new, state }
    }
}

#[derive(Clone)]
pub struct LinearScore {
    base: i64,
}

impl LinearScore {
    pub fn new(base: i64) -> Self {
        assert!(base > 0);
        Self { base }
    }

    pub fn linear(&self) -> i64 {
        self.base
    }

    pub fn percentage(&self, percent: usize) -> i64 {
        assert!(percent <= 100);
        self.base * (percent as i64) / 100
    }
}

impl Score<BlockBroadcastEntry> for LinearScore {
    fn execute(&self, entry: BlockBroadcastEntry) -> i64 {
        match entry.state {
            HandleState::Future => {
                if entry.new {
                    self.percentage(50)
                } else {
                    self.percentage(5)
                }
            }
            HandleState::Succ => {
                if entry.new {
                    self.linear()
                } else {
                    self.percentage(10)
                }
            }
            HandleState::Fail => -self.base,
        }
    }
}
