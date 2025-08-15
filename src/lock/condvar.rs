use crate::lock::mutex::*;
use atomic_wait::{wait, wake_all, wake_one};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

pub struct Condvar {
  counter: AtomicU32,
  num_waiters: AtomicU32,
}

impl Condvar {
  pub const fn new() -> Self {
    Self {
      counter: AtomicU32::new(0),
      num_waiters: AtomicU32::new(0),
    }
  }

  pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
    self.num_waiters.fetch_add(1, Ordering::Relaxed);
    let count_val = self.counter.load(Ordering::Relaxed);

    let mutex = guard.mutex;
    // unlock the mutex b dropping the guard
    // but remeber the mutex so we can lock it again
    drop(guard);

    // wait, but only if the counter hasn't changed since unlocking
    wait(&self.counter, count_val);

    self.num_waiters.fetch_sub(1, Ordering::Relaxed);
    mutex.lock()
  }

  pub fn notify_one(&self) {
    if self.num_waiters.load(Ordering::Relaxed) > 0 {
      self.counter.fetch_add(1, Ordering::Relaxed);
      wake_one(&self.counter);
    }
  }

  pub fn notify_all(&self) {
    if self.num_waiters.load(Ordering::Relaxed) > 0 {
      self.counter.fetch_add(1, Ordering::Relaxed);

      wake_all(&self.counter);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lock::mutex::Mutex;
  use std::thread;
  use std::time::{Duration, Instant};

  #[test]
  fn test_condvar() {
    let mutex = Mutex::new(0);
    let convar = Condvar::new();
    let mut wakeups = 0;
    thread::scope(|s| {
      let start = Instant::now();
      s.spawn(|| {
        thread::sleep(Duration::from_secs(1));
        *mutex.lock() = 123;
        convar.notify_one();

        // already notify one, don't notify again
        // don't effect the test time cost
        for _ in 0..1_000_000 {
          convar.notify_one();
        }
      });
      let mut guard = mutex.lock();
      while *guard < 100 {
        guard = convar.wait(guard);
        wakeups += 1;
      }
      dbg!(start.elapsed().as_millis())
    });
    dbg!(*mutex.lock(), wakeups);
  }
}
