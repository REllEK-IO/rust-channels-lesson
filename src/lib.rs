use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex}
};

// Flavours
// - Synchronous: Channel were send() can block. Limited capacity. Bounded
// -- Mutex + Condvar: VecDeque - Block if full
// -- Or with: Atomic VecDeque/Atomic Que - Update them atomically + thread::park + thread::Thread::notify
// - Asynchronous: Where send() cannot block. Unbounded
// -- Mutex + Condvar + VecDeque
// -- Or with: Mutex + Condvar + LinkedList, linked list of atomic VecDeque<T>
// -- Atomic block linked list, linked list of atomic VecDeque<T>
// --- Check this shit out in crossbeam
// - Rendezvous: Synchronous with capacity is 0. Used for thread synchronization.
// -- Can only send to a thread waiting. Used with a barrier
// - One-Shot: Channels you use once at any capacity. Only one call to send() in practice
// Examples: https://github.com/crossbeam-rs/crossbeam/tree/master/crossbeam-channel/src

//Arc - A thread-safe reference counting pointer. 'Arc' stands for 'Atomically Reference Counted'
// -- Used across thread boundaries
// -- Ensures that Sender and Receiver in this context are linked
//Mutex - A mutual exclusive primitive useful for protecting shared data
// -- Mutex lock returns guard which stops rewrite of value until released

// async/await

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);

        Sender { 
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        let was_last = inner.senders == 0;
        drop(inner);

        if was_last {
            self.shared.available.notify_one();
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(t);
        drop(inner);
        self.shared.available.notify_one();
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>,
}


impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        if let Some(t) = self.buffer.pop_front() {
            return Some(t);
        }

        loop {
            let mut inner = self.shared.inner.lock().unwrap();
            match inner.queue.pop_front() {
                Some(t) => {
                    // If only one receiver, we are taking all buffered items in a loop
                    if !inner.queue.is_empty() {
                        // Heyo swap function. No power of three here
                        std::mem::swap(&mut self.buffer, &mut inner.queue);
                    }
                    return Some(t)
                },
                None if inner.senders == 0 => return None,
                None => {
                    inner = self.shared.available.wait(inner).unwrap();
                }
            }
        }
    }
}

// Iterator of Receiver
impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}


struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize,
}
struct Shared<T> {
    inner: Mutex<Inner<T>>,
    available: Condvar,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::new(),
        senders: 1,
    };
    let shared = Shared {
        inner: Mutex::new(inner),
        available: Condvar::new(),
    };
    let shared = Arc::new(shared);
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared: shared.clone(),
            buffer: VecDeque::new(),
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_ping_pong() {
        let (mut tx, mut rx) = channel();
        tx.send(42);
        assert_eq!(rx.recv(), Some(42));
    }

    #[test]
    fn closed_sender() {
        let (tx, mut rx) = channel::<()>();
        // Drop the sender to close the receiver
        drop(tx);
        assert_eq!(rx.recv(), None);
    }
    #[test]
    fn closed_receiver() {
        let (mut tx, mut rx) = channel();

        drop(rx);
        tx.send(42);
        // THIS SHOULD PANIC, NEED TO BUILD OUT MEANS TO DO SO
    }
}