mod executor;
pub mod sender;

#[cfg(test)]
pub(crate) fn set_test_execute_delay_ms(delay_ms: u64) {
    executor::set_test_execute_delay_ms(delay_ms);
}

#[cfg(test)]
mod tests;
