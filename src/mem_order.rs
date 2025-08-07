mod eg;

#[cfg(test)]
pub mod tests {
  use std::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
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
}
