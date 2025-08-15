mod eg;

#[cfg(test)]
pub mod tests {
  use std::sync::atomic::{
    AtomicBool, AtomicUsize,
    Ordering::{self, Acquire, Release},
    compiler_fence,
  };
  use std::thread;
  use std::time::Duration;

  static mut DATA: u64 = 0;
  static READY: AtomicBool = AtomicBool::new(false);

  #[test]
  fn test_mem_order() {
    thread::spawn(|| {
      // Safety: Nothing else is accessing DATA,
      // because we haven't set the READY flag yet.
      unsafe { DATA = 123 };
      READY.store(true, Release); // Everything from before this store ..
    });
    while !READY.load(Acquire) {
      // .. is visible after this loads `true`.
      thread::sleep(Duration::from_millis(100));
      println!("waiting...");
    }
    // Safety: Nothing is mutating DATA, because READY is set.
    dbg!(unsafe { DATA });
  }

  #[test]
  fn test_relaxed() {
    use super::eg::{a, b};
    thread::scope(|s| {
      s.spawn(|| {
        a();
      });
      s.spawn(|| {
        b();
      });
    });
  }

  #[test]
  fn test_lock() {
    use super::eg::lock;
    thread::scope(|s| {
      for _ in 0..100 {
        s.spawn(lock);
      }
    });
  }

  #[test]
  fn test_fence() {
    use super::eg::order_fence;
    order_fence();
  }

  #[test]
  fn test_compiler_fence() {
    let locked = AtomicBool::new(false);
    let counter = AtomicUsize::new(0);
    thread::scope(|s| {
      for _ in 0..4 {
        s.spawn(|| {
          for _ in 0..1_000_000 {
            while locked.swap(true, Ordering::Relaxed) {}
            compiler_fence(Ordering::Acquire);

            let old = counter.load(Ordering::Relaxed);
            counter.store(old + 1, Ordering::Relaxed);

            compiler_fence(Ordering::Release);

            locked.store(false, Ordering::Relaxed);
          }
        });
      }
    });
    dbg!(counter.into_inner());
  }
}
