use std::cell::UnsafeCell;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};

use atomic_wait::{wait, wake_all, wake_one};

pub struct RwLock<T> {
  // the numbers of readers, or u32::MAX if write-locked.
  state: AtomicU32,
  // Incremented to wake up writers.
  writer_wait_counter: AtomicU32,
  val: UnsafeCell<T>,
}

pub struct ReadGuard<'a, T> {
  lock: &'a RwLock<T>,
}

impl<T> Deref for ReadGuard<'_, T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.lock.val.get() }
  }
}

impl<T> Drop for ReadGuard<'_, T> {
  fn drop(&mut self) {
    if self.lock.state.fetch_sub(1, Ordering::Release) == 1 {
      self
        .lock
        .writer_wait_counter
        .fetch_add(1, Ordering::Release);
      wake_one(&self.lock.writer_wait_counter);
    }
  }
}

pub struct WriteGuard<'a, T> {
  lock: &'a RwLock<T>,
}

impl<T> Deref for WriteGuard<'_, T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.lock.val.get() }
  }
}

impl<T> DerefMut for WriteGuard<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.lock.val.get() }
  }
}

impl<T> Drop for WriteGuard<'_, T> {
  fn drop(&mut self) {
    self.lock.state.store(0, Ordering::Relaxed);
    self
      .lock
      .writer_wait_counter
      .fetch_add(1, Ordering::Release);
    wake_all(&self.lock.state);
    wake_one(&self.lock.writer_wait_counter);
  }
}

impl<T> RwLock<T> {
  pub const fn new(val: T) -> Self {
    Self {
      state: AtomicU32::new(0),
      writer_wait_counter: AtomicU32::new(0),
      val: UnsafeCell::new(val),
    }
  }

  pub fn read(&self) -> ReadGuard<T> {
    let mut s = self.state.load(Ordering::Relaxed);
    loop {
      if s < u32::MAX {
        assert!(s != u32::MAX - 1, "Too many readers");
        match self
          .state
          .compare_exchange_weak(s, s + 1, Ordering::Acquire, Ordering::Relaxed)
        {
          Ok(_) => return ReadGuard { lock: self },
          Err(prev) => s = prev,
        }
      }
      if s == u32::MAX {
        wait(&self.state, u32::MAX);
        s = self.state.load(Ordering::Relaxed);
      }
    }
  }

  pub fn write(&self) -> WriteGuard<T> {
    while let Err(prev) =
      self
        .state
        .compare_exchange(0, u32::MAX, Ordering::Acquire, Ordering::Relaxed)
    {
      let w = self.writer_wait_counter.load(Ordering::Acquire);
      if self.state.load(Ordering::Relaxed) != 0 {
        // Wait for the RwLock is till locked, but only if
        // there have been no wake signals since we checked.
        wait(&self.writer_wait_counter, w);
      }
    }
    WriteGuard { lock: self }
  }
}
