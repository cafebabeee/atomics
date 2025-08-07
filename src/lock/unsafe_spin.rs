use std::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  sync::atomic::{
    AtomicBool,
    Ordering::{Relaxed, Release},
  },
};

pub struct UnsafeSpinLock<T> {
  locked: AtomicBool,
  value: UnsafeCell<T>,
}

unsafe impl<T> Sync for UnsafeSpinLock<T> where T: Send {}

pub struct Guard<'a, T> {
  lock: &'a UnsafeSpinLock<T>,
}

impl<T> UnsafeSpinLock<T> {
  pub const fn new(value: T) -> Self {
    UnsafeSpinLock {
      locked: AtomicBool::new(false),
      value: UnsafeCell::new(value),
    }
  }

  pub fn lock(&self) -> Guard<T> {
    while self
      .locked
      .compare_exchange_weak(false, true, Release, Relaxed)
      .is_err()
    {
      std::hint::spin_loop();
    }
    Guard { lock: self }
  }

  pub fn unlock(&self) {
    self.locked.store(false, Release);
  }
}

unsafe impl<T: Send> Send for Guard<'_, T> {}
unsafe impl<T: Sync> Sync for Guard<'_, T> {}

impl<T> Deref for Guard<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.lock.value.get() }
  }
}

impl<T> DerefMut for Guard<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.lock.value.get() }
  }
}

impl<'a, T> Drop for Guard<'a, T> {
  fn drop(&mut self) {
    self.lock.unlock();
  }
}
