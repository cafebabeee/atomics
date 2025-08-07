use std::sync::atomic::{AtomicBool, Ordering::Acquire, Ordering::Release};

pub struct SpinLock {
  lock: AtomicBool,
}

impl SpinLock {
  pub const fn new() -> Self {
    SpinLock {
      lock: AtomicBool::new(false),
    }
  }

  pub fn lock(&self) {
    while self.lock.swap(true, Acquire) {
      std::hint::spin_loop();
    }

    // while self
    //   .lock
    //   .compare_exchange_weak(false, true, Acquire, Relaxed)
    //   .is_err()
    // {
    //   std::hint::spin_loop();
    // }
  }

  pub fn unlock(&self) {
    self.lock.store(false, Release);
  }
}
