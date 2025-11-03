use std::sync::Mutex;

struct State {
    cache: Vec<i64>,
    gas_used: u64,
    next_calc_idx: usize,
}

pub struct GasTracker {
    state: Mutex<State>,
    gas_limit: u64,
}

impl GasTracker {
    pub fn new(total_txns: usize, gas_limit: u64) -> Self {
        Self {
            state: Mutex::new(State {
                cache: vec![-1; total_txns],
                gas_used: 0,
                next_calc_idx: 0,
            }),
            gas_limit,
        }
    }

    pub fn update(&self, idx: usize, gas_used: u64) -> bool {
        let mut state = self.state.lock().unwrap();
        state.cache[idx] = gas_used as i64;
        assert!(idx >= state.next_calc_idx);
        while state.next_calc_idx < state.cache.len() && state.cache[state.next_calc_idx] >= 0 {
            state.gas_used += state.cache[state.next_calc_idx] as u64;
            state.next_calc_idx += 1;
        }
        state.gas_used >= self.gas_limit
    }

    pub fn decrease_validation_idx(&self, idx: usize) {
        let mut state = self.state.lock().unwrap();

        // this method only called when some txn affects later txns, so next_calc_idx cannot be zero here
        while state.next_calc_idx > idx + 1 {
            state.gas_used -= state.cache[state.next_calc_idx - 1] as u64;
            state.next_calc_idx -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GasTracker;

    fn gas_used(tracker: &GasTracker) -> u64 {
        tracker.state.lock().unwrap().gas_used
    }

    #[test]
    fn test_update() {
        let tracker = GasTracker::new(5, 1000);
        assert_eq!(gas_used(&tracker), 0);

        assert_eq!(tracker.update(0, 200), false);
        assert_eq!(gas_used(&tracker), 200);

        assert_eq!(tracker.update(2, 300), false);
        assert_eq!(gas_used(&tracker), 200);

        assert_eq!(tracker.update(1, 400), false);
        assert_eq!(gas_used(&tracker), 900);

        assert_eq!(tracker.update(3, 200), true);
        assert_eq!(gas_used(&tracker), 1100);

        assert_eq!(tracker.update(4, 100), true);
        assert_eq!(gas_used(&tracker), 1200);
    }

    #[test]
    fn test_decrease_validation_idx() {
        let tracker = GasTracker::new(5, 1000);
        assert_eq!(gas_used(&tracker), 0);

        assert_eq!(tracker.update(0, 200), false);
        assert_eq!(tracker.update(1, 400), false);
        assert_eq!(tracker.update(2, 300), false);
        assert_eq!(tracker.update(3, 200), true);
        assert_eq!(gas_used(&tracker), 1100);

        tracker.decrease_validation_idx(2);
        assert_eq!(gas_used(&tracker), 900);

        tracker.decrease_validation_idx(3);
        assert_eq!(gas_used(&tracker), 900);

        tracker.decrease_validation_idx(1);
        assert_eq!(gas_used(&tracker), 600);

        tracker.decrease_validation_idx(0);
        assert_eq!(gas_used(&tracker), 200);
    }
}
