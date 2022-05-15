#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Error {
    UnReachable,
    CircuitError,
}


pub(crate) type Circuit = dyn Fn() -> Result<String, Error>;

pub(crate) fn fail_after(threshold: usize) -> Box<Circuit> {
    let cnt = std::cell::RefCell::new(0);
    let f = move || {
        let mut c = cnt.borrow_mut();
        *c += 1;
        if *c > threshold {
            return Err(Error::CircuitError);
        } else {
            return Ok("ok".to_owned());
        }
    };
    Box::new(f)
}

fn breaker(circuit: Box<Circuit>, failure_threshold: u64) -> Box<Circuit> {
    let failures = std::cell::RefCell::new(0isize);

    let f = move || {
        let d = *failures.borrow() - failure_threshold as isize;
        if d >= 0 {
            return Err(Error::UnReachable);
        }

        let r: Result<String, Error> = circuit();
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
    return Box::new(f);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_after() {
        let f = fail_after(5);
        for i in 0..6 {
            let r: Result<String, Error> = f();
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
            let r: Result<String, Error> = b();
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