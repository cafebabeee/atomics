pub(crate) mod condvar;
pub(crate) mod mutex;
mod rwlock;
mod spin;
mod unsafe_spin;

#[cfg(test)]
mod tests {
  use super::unsafe_spin::UnsafeSpinLock;
  use std::thread;

  #[test]
  fn test_spinlock() {
    let x = UnsafeSpinLock::new(Vec::new());
    thread::scope(|s| {
      s.spawn(|| x.lock().push(1));
      s.spawn(|| {
        let mut g = x.lock();
        g.push(2);
        // drop(g); // explicitly unlock
        g.push(2);
      });
    });
    let g = x.lock();
    assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
  }
}
