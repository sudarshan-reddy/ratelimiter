use std::ops::Add;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[allow(dead_code)]
struct State {
    last: Option<Instant>,
    sleep_for: Duration,
}

impl Default for State {
    fn default() -> Self {
        State { last: Some(Instant::now()), sleep_for: Duration::new(0, 0) }
    }
}

#[allow(dead_code)]
pub struct LeakyBucket {
    state: AtomicPtr<State>,

    per_request: Duration,
    max_slack: Duration,
}

#[allow(dead_code)]
impl LeakyBucket {
    pub fn new(rate: u64) -> Self {
        let mut initial_state = State { last: None, ..State::default() };

        let atomic_ptr = AtomicPtr::new(&mut initial_state);
        LeakyBucket {
            state: atomic_ptr,
            per_request: Duration::new(1 / rate, 0),
            max_slack: Duration::new(10 / rate, 0),
        }
    }
}

#[allow(dead_code)]
impl crate::Limiter for LeakyBucket {
    fn take(&self) -> Option<Instant> {
        let mut new_state = State::default();
        let mut taken = false;

        while !taken {
            let prev_state = self.state.load(Ordering::Acquire);
            new_state = State::default();

            let prev_state_last: Option<Instant>;

            unsafe {
                prev_state_last = (*prev_state).last;
            }

            // TODO: see if this can be converted to a combinator
            // or a match
            if prev_state_last == None {
                let ret_val = self.state.compare_and_swap(
                    prev_state,
                    &mut new_state,
                    Ordering::Relaxed,
                );
                taken = ret_val == prev_state;
                continue;
            }

            // Can this be negative?
            new_state.sleep_for += self.per_request
                - Instant::now().duration_since(prev_state_last.unwrap());

            if new_state.sleep_for < self.max_slack {
                new_state.sleep_for = self.max_slack;
            }

            new_state.last =
                Some(new_state.last.unwrap().add(new_state.sleep_for));
        }
        thread::sleep(new_state.sleep_for);
        new_state.last
    }
}
