use std::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicU32, Ordering},
};

use atomic_wait::{wait, wake_all, wake_one};

pub struct Mutex<T> {
  // 0： unlocked
  // 1: locked, no other thread waiting
  // 2: locked, other threads are waiting
  state: AtomicU32,
  value: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
  #[inline]
  pub const fn new(val: T) -> Self {
    Self {
      state: AtomicU32::new(0),
      value: UnsafeCell::new(val),
    }
  }

  pub fn lock(&self) -> MutexGuard<'_, T> {
    // while self.state.swap(1, Ordering::Acquire) == 1 {
    //   wait(&self.state, 1);
    // }
    if self
      .state
      .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
      .is_err()
    {
      //while self.state.swap(2, Ordering::Acquire) != 0 {
      // wait(&self.state, 2);
      //}
      // The lock was already contended
      lock_contended(&self.state);
    }
    MutexGuard { mutex: self }
  }
}

fn lock_contended(state: &AtomicU32) {
  let mut spin_count = 0;
  while state.load(Ordering::Relaxed) == 1 && spin_count < 100 {
    std::hint::spin_loop();
    spin_count += 1;
  }
  if state
    .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
    .is_ok()
  {
    return;
  }

  while state.swap(2, Ordering::Acquire) != 0 {
    wait(state, 2);
  }
}

pub struct MutexGuard<'a, T> {
  pub(super) mutex: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.mutex.value.get() }
  }
}

impl<T> DerefMut for MutexGuard<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.mutex.value.get() }
  }
}

impl<T> Drop for MutexGuard<'_, T> {
  fn drop(&mut self) {
    if self.mutex.state.swap(0, Ordering::Release) == 2 {
      wake_one(&self.mutex.state);
    }
  }
}

#[cfg(test)]
mod tests {

  use std::{thread, time::Instant};

  use super::*;

  #[test]
  fn test_mutex() {
    let mut m = Mutex::new(5);
    *m.lock() += 1;
    *m.lock() += 2;
    thread::scope(|s| {
      s.spawn(|| {
        *m.lock() += 3;
      });
      dbg!(*m.lock());
    });
    dbg!(*m.lock());
  }

  #[test]
  fn mutex_benckmark() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    for _ in 0..5_000_000 {
      *m.lock() += 1;
    }
    dbg!(*m.lock(), start.elapsed());
  }

  #[test]
  fn mutil_mutex_benchmark() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();

    thread::scope(|s| {
      for _ in 0..4 {
        s.spawn(|| {
          for _ in 0..5_000_000 {
            *m.lock() += 1;
          }
        });
      }
      dbg!(*m.lock());
    });
    dbg!(*m.lock(), start.elapsed());
  }
}
