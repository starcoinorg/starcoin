mod inbound;
mod outbound;
mod pool;
mod proto;
mod sync;
#[cfg(test)]
mod tests;

pub trait Synchronizer {}
