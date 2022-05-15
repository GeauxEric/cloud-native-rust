use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time;
use std::time::Duration;

use crate::ch04::circuit_breaker::{Circuit, Error};

/// On each call of the outer function—regardless of its outcome — a time interval is set.
/// Any subsequent call made before that time interval expires is ignored;
/// any call made afterwards is passed along to the inner function.
fn debounce_first(circuit: Circuit, d: Duration) -> Circuit {
    let t = std::cell::RefCell::new(time::UNIX_EPOCH);
    let cache = std::cell::RefCell::new(Err(Error::CircuitError));

    let c1 = circuit.clone();
    let f = move || {
        let mut threshold = t.borrow_mut();
        return if time::SystemTime::now() < *threshold {
            println!("use cache");
            cache.borrow().clone()
        } else {
            println!("calling inner function");
            let result: Result<String, Error> = c1.lock().unwrap()();
            *cache.borrow_mut() = result.clone();
            *threshold = time::SystemTime::now().add(d);
            result
        };
    };
    Arc::new(Mutex::new(f))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use crate::ch04::circuit_breaker::{fail_after, Error};
    use crate::ch04::debounce_first::debounce_first;

    #[test]
    fn test_debounce_first() {
        let c = fail_after(1);
        let d = debounce_first(c, Duration::from_secs(1));
        for _ in 0..10 {
            let r = d.lock().unwrap()();
            assert!(r.is_ok());
        }
    }

    #[test]
    fn test_debounce_first_data_race() {
        let c = fail_after(1);
        let d = debounce_first(c, Duration::from_secs(1));
        let mut results = vec![];
        for _ in 0..10 {
            let d1 = Arc::clone(&d);
            let h = thread::spawn(move || {
                let r: Result<String, Error> = d1.lock().unwrap()();
                r
            });
            results.push(h)
        }

        for result in results {
            let r = result.join().unwrap();
            assert!(r.is_ok())
        }
    }
}
