pub mod faucet;
pub mod web;

#[macro_export]
macro_rules! unwrap_or_return {
    ($e:expr, $r:expr) => {
        match $e {
            Ok(e) => e,
            Err(_e) => return $r,
        }
    };
}
