mod executor;
pub mod parallel_info_service;
pub mod sender;

#[cfg(test)]
pub(crate) fn set_test_execute_delay_ms(delay_ms: u64) {
    executor::set_test_execute_delay_ms(delay_ms);
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) fn set_test_assume_parents_ready(ready: bool) {
    executor::set_test_assume_parents_ready(ready);
}

#[cfg(test)]
pub(crate) fn reset_test_parent_check_invocations() {
    executor::reset_test_parent_check_invocations();
}

#[cfg(test)]
pub(crate) fn test_parent_check_invocations() -> u64 {
    executor::test_parent_check_invocations()
}
