use std::sync::Mutex;

struct State {
    // stores gas used for each transaction
    // -1 means not yet validated
    //  0 means validated but discarded
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

    pub fn update_and_check_reach_gas_limit(&self, idx: usize, gas_used: u64) -> bool {
        let mut state = self.state.lock().unwrap();

        // If idx < next_calc_idx, it means this transaction is being re-validated
        // after a decrease_validation_idx call. We need to update the cache and
        // recalculate gas_used from this index.
        if idx < state.next_calc_idx {
            // Update the cache
            state.cache[idx] = gas_used as i64;

            // Recalculate gas_used from index 0
            state.gas_used = 0;
            state.next_calc_idx = 0;

            // Recalculate up to the first unvalidated transaction
            while state.next_calc_idx < state.cache.len() && state.cache[state.next_calc_idx] >= 0 {
                state.gas_used += state.cache[state.next_calc_idx] as u64;
                state.next_calc_idx += 1;
            }
        } else {
            // Normal case: idx >= next_calc_idx
            state.cache[idx] = gas_used as i64;

            // Advance next_calc_idx if possible
            while state.next_calc_idx < state.cache.len() && state.cache[state.next_calc_idx] >= 0 {
                state.gas_used += state.cache[state.next_calc_idx] as u64;
                state.next_calc_idx += 1;
            }
        }

        // Return true if gas limit is exceeded (> not >=, to allow exact limit)
        state.gas_used > self.gas_limit
    }

    pub fn decrease_validation_idx(&self, idx: usize) {
        let mut state = self.state.lock().unwrap();

        // this method only called when some txn affects later txns, so next_calc_idx cannot be zero here
        while state.next_calc_idx > idx + 1 {
            state.gas_used -= state.cache[state.next_calc_idx - 1] as u64;
            state.next_calc_idx -= 1;
        }
    }

    pub fn first_exceeding_index(&self) -> usize {
        let state = self.state.lock().unwrap();

        // If total gas used is within or equal to limit, all validated transactions can execute
        if state.gas_used <= self.gas_limit {
            return state.cache.len();
        }

        // gas_used > limit: find the first transaction that causes exceeding
        let mut cumulative_gas = 0u64;
        for i in 0..state.next_calc_idx {
            cumulative_gas += state.cache[i] as u64;
            if cumulative_gas > self.gas_limit {
                return i;
            }
        }

        state.cache.len()
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

        // Out-of-order updates
        assert!(!tracker.update_and_check_reach_gas_limit(0, 200));
        assert_eq!(gas_used(&tracker), 200);

        assert!(!tracker.update_and_check_reach_gas_limit(2, 300));
        assert_eq!(gas_used(&tracker), 200);

        assert!(!tracker.update_and_check_reach_gas_limit(1, 400));
        assert_eq!(gas_used(&tracker), 900);

        // Exceed limit
        assert!(tracker.update_and_check_reach_gas_limit(3, 200));
        assert_eq!(gas_used(&tracker), 1100);
    }

    #[test]
    fn test_decrease_validation_idx() {
        let tracker = GasTracker::new(5, 1000);

        assert!(!tracker.update_and_check_reach_gas_limit(0, 200));
        assert!(!tracker.update_and_check_reach_gas_limit(1, 400));
        assert!(!tracker.update_and_check_reach_gas_limit(2, 300));
        assert!(tracker.update_and_check_reach_gas_limit(3, 200));
        assert_eq!(gas_used(&tracker), 1100);

        tracker.decrease_validation_idx(2);
        assert_eq!(gas_used(&tracker), 900);

        tracker.decrease_validation_idx(1);
        assert_eq!(gas_used(&tracker), 600);
    }

    #[test]
    fn test_first_exceeding_index() {
        let tracker = GasTracker::new(5, 1000);

        // No transactions
        assert_eq!(tracker.first_exceeding_index(), 5);

        // Under limit
        tracker.update_and_check_reach_gas_limit(0, 200);
        tracker.update_and_check_reach_gas_limit(1, 400);
        tracker.update_and_check_reach_gas_limit(2, 300);
        assert_eq!(tracker.first_exceeding_index(), 5);

        // Exceed limit
        tracker.update_and_check_reach_gas_limit(3, 200);
        assert_eq!(tracker.first_exceeding_index(), 3);

        // Decrease validation
        tracker.decrease_validation_idx(2);
        assert_eq!(tracker.first_exceeding_index(), 5);
    }

    #[test]
    fn test_first_exceeding_index_edge_cases() {
        // Exact limit
        let tracker = GasTracker::new(5, 1000);
        tracker.update_and_check_reach_gas_limit(0, 500);
        tracker.update_and_check_reach_gas_limit(1, 500);
        assert_eq!(tracker.first_exceeding_index(), 5);

        tracker.update_and_check_reach_gas_limit(2, 1);
        assert_eq!(tracker.first_exceeding_index(), 2);

        // Unvalidated gap
        let tracker2 = GasTracker::new(5, 1000);
        tracker2.update_and_check_reach_gas_limit(0, 200);
        tracker2.update_and_check_reach_gas_limit(1, 400);
        tracker2.update_and_check_reach_gas_limit(3, 300);
        tracker2.update_and_check_reach_gas_limit(4, 200);
        assert_eq!(tracker2.first_exceeding_index(), 5);

        // First transaction exceeds limit
        let tracker3 = GasTracker::new(3, 100);
        assert!(tracker3.update_and_check_reach_gas_limit(0, 200));
        assert_eq!(tracker3.first_exceeding_index(), 0);

        // Zero gas transactions
        let tracker4 = GasTracker::new(5, 1000);
        tracker4.update_and_check_reach_gas_limit(0, 0);
        tracker4.update_and_check_reach_gas_limit(1, 500);
        tracker4.update_and_check_reach_gas_limit(2, 0);
        tracker4.update_and_check_reach_gas_limit(3, 501);
        assert_eq!(tracker4.first_exceeding_index(), 3);
    }

    #[test]
    fn test_decrease_validation_edge_cases() {
        let tracker = GasTracker::new(5, 1000);
        tracker.update_and_check_reach_gas_limit(0, 200);
        tracker.update_and_check_reach_gas_limit(1, 300);
        tracker.update_and_check_reach_gas_limit(2, 400);

        // Decrease to index 0
        tracker.decrease_validation_idx(0);
        assert_eq!(gas_used(&tracker), 200);

        // Decrease when already at target
        tracker.decrease_validation_idx(0);
        assert_eq!(gas_used(&tracker), 200);
    }
}
