#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use std::sync::{Arc, Condvar, Mutex};

//Arc - A thread-safe reference counting pointer. 'Arc' stands for 'Atomically Reference Counted'
// -- Used across thread boundaries
// -- Ensures that Sender and Receiver in this context are linked
//Mutex - A mutual exclusive primitive useful for protecting shared data
// -- Mutex lock returns guard which stops rewrite of value until released

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

struct Inner<T> {
    queue: Mutex<Vec<T>>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::default(),
    };
    let inner = Arc::new(inner);
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver {
            inner: inner.clone(),
        }
    )
}