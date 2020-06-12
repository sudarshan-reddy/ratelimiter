use std::sync::atomic::AtomicPtr;
use std::time::{Duration, Instant};

#[allow(dead_code)]
struct State {
    last: Instant,
    sleep_for: Duration,
}

#[allow(dead_code)]
pub struct Leaky {
    state: AtomicPtr<State>,

    per_request: Duration,
    max_slack: Duration,
}

#[allow(dead_code)]
impl Leaky {
    pub fn new(rate: u64) -> Self {
        let initial_state = &mut State {
            last: Instant::now(),
            sleep_for: Duration::new(0, 0),
        };

        let atomic_ptr = AtomicPtr::new(initial_state);
        Leaky {
            state: atomic_ptr,
            per_request: Duration::new(1 / rate, 0),
            max_slack: Duration::new(10 / rate, 0),
        }
    }
}
