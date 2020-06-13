#[allow(dead_code)]
use std::time::Instant;
mod leaky;

pub trait Limiter {
    fn take(&self) -> Option<Instant>;
}
