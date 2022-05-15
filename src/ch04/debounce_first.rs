use std::ops::Add;
use std::time;
use std::time::Duration;

use crate::ch04::circuitbreaker::{Circuit, Error};

/// On each call of the outer function—regardless of its outcome — a time interval is set.
/// Any subsequent call made before that time interval expires is ignored;
/// any call made afterwards is passed along to the inner function.
fn debounce_first(circuit: Box<Circuit>, d: Duration) -> Box<Circuit> {
    let t = std::cell::RefCell::new(time::UNIX_EPOCH);
    let cache = std::cell::RefCell::new(Err(Error::CircuitError));

    let f = move || {
        let mut threshold = t.borrow_mut();
        return if time::SystemTime::now() < *threshold {
            println!("use cache");
            cache.borrow().clone()
        } else {
            println!("calling inner function");
            let result: Result<String, Error> = circuit();
            *cache.borrow_mut() = result.clone();
            *threshold = time::SystemTime::now().add(d);
            result
        };
    };
    return Box::new(f);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::ch04::circuitbreaker::fail_after;
    use crate::ch04::debounce_first::debounce_first;

    #[test]
    fn test_debounce_first() {
        let c = fail_after(1);
        let d = debounce_first(c, Duration::from_secs(1));
        for _ in 0..10 {
            let r = d();
            assert!(r.is_ok());
        }
    }
}