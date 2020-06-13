use std::time::Instant;
pub mod leaky;

pub trait Limiter {
    fn take(&self) -> Option<Instant>;
}
