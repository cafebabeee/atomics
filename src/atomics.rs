pub mod eg;
#[cfg(test)]
mod test {
  use crate::atomics::eg;
  use std::{
    hint::black_box,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Instant,
  };
  #[test]
  pub fn test_process() {
    eg::process();
  }

  #[test]
  pub fn test_sync() {
    eg::sync();
  }

  #[test]
  pub fn test_lazy_init() {
    eg::lazy_init();
  }

  #[test]
  pub fn test_allocate_new_id() {
    thread::scope(|s| {
      for _ in 0..10 {
        s.spawn(|| {
          for _ in 0..100 {
            eg::allocate_new_id();
          }
        });
      }
    });
    eg::allocate_new_id();
  }

  #[test]
  fn test_cost() {
    static A: AtomicU64 = AtomicU64::new(0);
    black_box(&A);
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
      black_box(A.load(Ordering::Relaxed));
    }
    dbg!(start.elapsed().as_millis());
  }

  #[test]
  fn test_cost_store() {
    static A: AtomicU64 = AtomicU64::new(0);
    thread::spawn(|| {
      loop {
        A.store(0, Ordering::Relaxed);
      }
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
      A.load(Ordering::Relaxed);
    }
    dbg!(start.elapsed().as_millis());
  }

  #[test]
  fn test_cost_mutil() {
    static A: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
    thread::spawn(|| {
      loop {
        A[0].store(0, Ordering::Relaxed);
        A[2].store(0, Ordering::Relaxed);
      }
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
      A[1].load(Ordering::Relaxed);
    }
    dbg!(start.elapsed().as_millis());
  }

  #[test]
  fn test_cost_aligned() {
    #[repr(align(64))]
    struct Aligned(AtomicU64);

    static A: [Aligned; 3] = [
      Aligned(AtomicU64::new(0)),
      Aligned(AtomicU64::new(0)),
      Aligned(AtomicU64::new(0)),
    ];

    thread::spawn(|| {
      loop {
        A[0].0.store(0, Ordering::Relaxed);
        A[2].0.store(0, Ordering::Relaxed);
      }
    });

    let start = Instant::now();
    for _ in 0..1_000_000_000 {
      A[1].0.load(Ordering::Relaxed);
    }
    dbg!(start.elapsed().as_millis());
  }
}
