use std::sync::Arc;
use std::sync::Mutex;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Error {
    UnReachable,
    CircuitError,
}

pub(crate) type Circuit = Arc<Mutex<dyn Fn() -> Result<String, Error> + Send + 'static>>;

pub(crate) fn fail_after(threshold: usize) -> Circuit {
    let cnt = Arc::new(Mutex::new(0));
    let f = move || {
        let mut c = cnt.lock().unwrap();
        *c += 1;
        if *c > threshold {
            Err(Error::CircuitError)
        } else {
            Ok("ok".to_owned())
        }
    };
    Arc::new(Mutex::new(f))
}

fn breaker(circuit: Circuit, failure_threshold: u64) -> Circuit {
    let failures = std::cell::RefCell::new(0isize);

    let f = move || {
        let d = *failures.borrow() - failure_threshold as isize;
        if d >= 0 {
            return Err(Error::UnReachable);
        }

        let c = circuit.lock().unwrap();
        let r: Result<String, Error> = c();
        return match r {
            Ok(s) => {
                let mut f = failures.borrow_mut();
                *f = 0;
                Ok(s)
            }
            Err(e) => {
                let mut f = failures.borrow_mut();
                *f += 1;
                Err(e)
            }
        };
    };
    Arc::new(Mutex::new(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_after() {
        let f = fail_after(5);
        for i in 0..6 {
            let r: Result<String, Error> = f.lock().unwrap()();
            if i < 5 {
                assert!(r.is_ok())
            } else {
                assert!(r.is_err())
            }
        }
    }

    #[test]
    fn test_breaker() {
        let c = fail_after(5);
        let b = breaker(c, 1);
        for i in 0..9 {
            let r: Result<String, Error> = b.lock().unwrap()();
            if i < 5 {
                assert!(r.is_ok())
            } else if i == 5 {
                assert_eq!(r, Err(Error::CircuitError))
            } else {
                assert_eq!(r, Err(Error::UnReachable))
            }
        }
    }
}
