use std::{
  collections::VecDeque,
  sync::{Condvar, Mutex},
};

pub struct Channel<T> {
  queue: Mutex<VecDeque<T>>,
  ready: Condvar,
}

impl<T> Channel<T> {
  pub const fn new() -> Self {
    Channel {
      queue: Mutex::new(VecDeque::new()),
      ready: Condvar::new(),
    }
  }

  pub fn send(&self, val: T) {
    self.queue.lock().unwrap().push_back(val);
    self.ready.notify_one();
  }

  pub fn receive(&self) -> T {
    let mut q = self.queue.lock().unwrap();
    loop {
      if let Some(val) = q.pop_front() {
        return val;
      }
      q = self.ready.wait(q).unwrap();
    }
  }
}
