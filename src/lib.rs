#[allow(dead_code)]
use std::time::Instant;
mod leaky;

pub trait Limiter {
    fn take(&self) -> Option<Instant>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        std::time::Instant::now();
    }
}
