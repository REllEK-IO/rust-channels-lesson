use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex}
};

//Arc - A thread-safe reference counting pointer. 'Arc' stands for 'Atomically Reference Counted'
// -- Used across thread boundaries
// -- Ensures that Sender and Receiver in this context are linked
//Mutex - A mutual exclusive primitive useful for protecting shared data
// -- Mutex lock returns guard which stops rewrite of value until released

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender { 
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        let mut queue = self.inner.queue.lock().unwrap();
        queue.push_back(t);
        drop(queue);
        self.inner.available.notify_one();
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}


impl<T> Receiver<T> {
    pub fn recv(&mut self) -> T {
        loop {
            let mut queue = self.inner.queue.lock().unwrap();
            match queue.pop_front() {
                Some(t) => return t,
                None => {
                    queue = self.inner.available.wait(queue).unwrap();
                }
            }
        }
    }
}

struct Inner<T> {
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
}

pub fn _channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: Mutex::default(),
        available: Condvar::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_ping_pong() {
        let (mut tx, mut rx) = _channel();
        tx.send(42);
        assert_eq!(rx.recv(), 42);
    }
}