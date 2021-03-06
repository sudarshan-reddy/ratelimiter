use std::ops::Add;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;
use std::time::{Duration, Instant};

struct State {
    last: Option<Instant>,
    sleep_for: Duration,
}

impl Default for State {
    fn default() -> Self {
        State { last: Some(Instant::now()), sleep_for: Duration::new(0, 0) }
    }
}

pub struct LeakyBucket {
    state: AtomicPtr<State>,

    per_request: Duration,

    #[allow(dead_code)]
    max_slack: Duration,
}

impl LeakyBucket {
    pub fn new(rate: u64) -> Self {
        let mut initial_state = State { last: None, ..State::default() };

        let atomic_ptr = AtomicPtr::new(&mut initial_state);

        LeakyBucket {
            state: atomic_ptr,
            per_request: Duration::new(1 / rate, 0),
            max_slack: Duration::new(0, 0),
        }
    }
}

impl crate::Limiter for LeakyBucket {
    fn take(&self) -> Option<Instant> {
        let mut new_state = State::default();
        let mut taken = false;

        while !taken {
            let prev_state = self.state.load(Ordering::Acquire);
            new_state = State::default();

            let prev_state_last_opt: Option<Instant>;

            unsafe {
                prev_state_last_opt = (*prev_state).last;
            }

            taken = match prev_state_last_opt {
                Some(prev_state_last) => {
                    let since = Instant::now().duration_since(prev_state_last);
                    if self.per_request > since {
                        new_state.sleep_for += self.per_request - since;
                    }
                    if new_state.sleep_for < self.max_slack {
                        new_state.sleep_for = self.max_slack;
                    }

                    new_state.last =
                        Some(new_state.last.unwrap().add(new_state.sleep_for));

                    let ret_val = self.state.compare_and_swap(
                        prev_state,
                        &mut new_state,
                        Ordering::Release,
                    );
                    ret_val == prev_state
                }

                None => {
                    let ret_val = self.state.compare_and_swap(
                        prev_state,
                        &mut new_state,
                        Ordering::Release,
                    );
                    ret_val == prev_state
                }
            };
        }
        thread::sleep(new_state.sleep_for);
        new_state.last
    }
}

#[cfg(test)]
mod tests {
    use crate::leaky::LeakyBucket;
    use crate::Limiter;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};
    #[test]
    fn test_unlimited() {
        let l = LeakyBucket::new(1);
        let now = Instant::now();
        let mut prev = Instant::now();
        for i in 0..10 {
            let t = l.take();
            println!("{:?}, {:?}", t, prev);
            let elapsed_in_millis =
                t.unwrap().duration_since(prev).as_millis();
            println!("{}, {}", i, elapsed_in_millis);
            prev = t.unwrap();
            // 1010 milliseconds because the time based leaky bucket could
            // be 10% higher or lower.
            assert_eq!(elapsed_in_millis < 1010, true);
        }

        assert_eq!(now.elapsed() > Duration::new(9, 0), true);
    }

    //#[test]
    //fn test_rate_limiting() {
    //    let l = LeakyBucket::new(1);

    //    let safe_l = Arc::new(Mutex::new(l));

    //    let handles = (0..4)
    //        .into_iter()
    //        .map(|_| {
    //            let data = Arc::clone(&safe_l);
    //            thread::spawn(move || {
    //                let l = data.lock().unwrap();
    //                let now = Instant::now();
    //                let t = l.take();
    //                println!("{}", t.unwrap().duration_since(now).as_millis());
    //            })
    //        })
    //        .collect::<Vec<thread::JoinHandle<_>>>();

    //    for thread in handles {
    //        thread.join().unwrap();
    //    }
    //}
}
