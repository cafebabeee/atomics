use std::{
  cell::UnsafeCell,
  mem::ManuallyDrop,
  ops::Deref,
  ptr::NonNull,
  sync::atomic::{AtomicUsize, Ordering, fence},
  usize,
};

struct ArcData<T> {
  data: UnsafeCell<ManuallyDrop<T>>,
  /// numbers of `Arc`s
  ref_count: AtomicUsize,
  /// numbers of `Weak`s, plus one if there are any `Arc`s
  alloc_ref_count: AtomicUsize,
}

pub struct Weak<T> {
  ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Weak<T> {}
unsafe impl<T: Send + Sync> Sync for Weak<T> {}

pub struct Arc<T> {
  ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
  pub fn new(data: T) -> Self {
    Arc {
      ptr: NonNull::from(Box::leak(Box::new(ArcData {
        data: UnsafeCell::new(ManuallyDrop::new(data)),
        ref_count: AtomicUsize::new(1),
        alloc_ref_count: AtomicUsize::new(1),
      }))),
    }
  }

  fn data(&self) -> &ArcData<T> {
    unsafe { self.ptr.as_ref() }
  }

  pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
    if arc
      .data()
      .alloc_ref_count
      .compare_exchange(1, usize::MAX, Ordering::Acquire, Ordering::Relaxed)
      .is_err()
    {
      return None;
    }
    let is_unique = arc.data().ref_count.load(Ordering::Relaxed) == 1;
    arc.data().alloc_ref_count.store(1, Ordering::Release);
    if !is_unique {
      return None;
    }
    fence(Ordering::Acquire);
    unsafe { Some(&mut *arc.data().data.get()) }
  }

  pub fn downgrade(arc: &Self) -> Weak<T> {
    let mut count = arc.data().alloc_ref_count.load(Ordering::Relaxed);

    loop {
      if count == usize::MAX {
        std::hint::spin_loop();
        count = arc.data().alloc_ref_count.load(Ordering::Relaxed);
        continue;
      }
      assert!(count < usize::MAX - 1);
      if let Err(e) = arc.data().alloc_ref_count.compare_exchange_weak(
        count,
        count + 1,
        Ordering::Acquire,
        Ordering::Relaxed,
      ) {
        count = e;
        continue;
      }
      return Weak { ptr: arc.ptr };
    }
  }
}

impl<T> Deref for Arc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.data().data.get() }
  }
}

impl<T> Clone for Arc<T> {
  fn clone(&self) -> Self {
    if self.data().ref_count.fetch_add(1, Ordering::Relaxed) >= usize::MAX / 2 {
      std::process::abort();
    }
    Arc { ptr: self.ptr }
  }
}

impl<T> Drop for Arc<T> {
  fn drop(&mut self) {
    if self.data().ref_count.fetch_sub(1, Ordering::Release) == 1 {
      fence(Ordering::Acquire);
      unsafe {
        ManuallyDrop::drop(&mut *self.data().data.get());
      }
      drop(Weak { ptr: self.ptr });
    }
  }
}

impl<T> Weak<T> {
  pub fn data(&self) -> &ArcData<T> {
    unsafe { self.ptr.as_ref() }
  }

  pub fn upgrade(&self) -> Option<Arc<T>> {
    let mut n = self.data().ref_count.load(Ordering::Relaxed);
    loop {
      if n == 0 {
        return None;
      }
      assert!(n < usize::MAX);

      if let Err(e) =
        self
          .data()
          .ref_count
          .compare_exchange_weak(n, n + 1, Ordering::Relaxed, Ordering::Relaxed)
      {
        n = e;
        continue;
      }
      return Some(Arc { ptr: self.ptr });
    }
  }
}

impl<T> Clone for Weak<T> {
  fn clone(&self) -> Self {
    if self.data().alloc_ref_count.fetch_add(1, Ordering::Relaxed) >= usize::MAX / 2 {
      std::process::abort();
    }
    Self { ptr: self.ptr }
  }
}

impl<T> Drop for Weak<T> {
  fn drop(&mut self) {
    if self.data().alloc_ref_count.fetch_sub(1, Ordering::Release) == 1 {
      fence(Ordering::Acquire);
      unsafe {
        drop(Box::from_raw(self.ptr.as_ptr()));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Arc;
  use std::sync::atomic::AtomicUsize;
  use std::sync::atomic::Ordering::Relaxed;
  use std::thread;
  #[test]
  fn test_weak() {
    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);
    struct DetectDrop;
    impl Drop for DetectDrop {
      fn drop(&mut self) {
        NUM_DROPS.fetch_add(1, Relaxed);
      }
    }
    // Create an Arc with two weak pointers.
    let x = Arc::new(("hello", DetectDrop));
    let y = Arc::downgrade(&x);
    let z = Arc::downgrade(&x);
    let t = thread::spawn(move || {
      // Weak pointer should be upgradable at this point.
      let y = y.upgrade().unwrap();
      dbg!(y.0)
    });
    dbg!(x.0);
    t.join().unwrap();
    // The data shouldn't be dropped yet,
    // and the weak pointer should be upgradable.
    dbg!(NUM_DROPS.load(Relaxed));
    dbg!(z.upgrade().is_some());
    //let w = x.clone();

    drop(x);
    // Now, the data should be dropped, and the
    // weak pointer should no longer be upgradable.
    dbg!(NUM_DROPS.load(Relaxed));
    dbg!(z.upgrade().is_none());
  }
}
