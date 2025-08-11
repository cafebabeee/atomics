use std::{
  cell::UnsafeCell,
  ops::Deref,
  ptr::NonNull,
  sync::atomic::{AtomicUsize, Ordering, fence},
};

struct ArcData<T> {
  data: UnsafeCell<Option<T>>,
  ref_count: AtomicUsize,
  alloc_ref_count: AtomicUsize,
}

pub struct Weak<T> {
  ptr: NonNull<ArcData<T>>,
}

pub struct Arc<T> {
  weak: Weak<T>,
}

impl<T> Arc<T> {
  pub fn new(data: T) -> Self {
    Arc {
      weak: Weak {
        ptr: NonNull::from(Box::leak(Box::new(ArcData {
          data: UnsafeCell::new(Some(data)),
          ref_count: AtomicUsize::new(1),
          alloc_ref_count: AtomicUsize::new(1),
        }))),
      },
    }
  }

  pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
    if arc.weak.data().ref_count.load(Ordering::Relaxed) == 1 {
      fence(Ordering::Acquire);
      unsafe { Some(arc.weak.ptr.as_mut().data.get_mut().as_mut().unwrap()) }
    } else {
      None
    }
  }

  pub fn downgrade(arc: &Self) -> Weak<T> {
    arc.weak.clone()
  }
}

unsafe impl<T: Send + Sync> Send for Weak<T> {}
unsafe impl<T: Send + Sync> Sync for Weak<T> {}

impl<T> Deref for Arc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { (*self.weak.data().data.get()).as_ref().unwrap() }
  }
}

impl<T> Clone for Arc<T> {
  fn clone(&self) -> Self {
    let weak = self.weak.clone();
    if weak.data().ref_count.fetch_add(1, Ordering::Relaxed) >= usize::MAX / 2 {
      std::process::abort();
    }
    Arc { weak }
  }
}

impl<T> Drop for Arc<T> {
  fn drop(&mut self) {
    if self.weak.data().ref_count.fetch_sub(1, Ordering::Release) == 1 {
      fence(Ordering::Acquire);
      unsafe {
        std::mem::swap(&mut (*self.weak.data().data.get()), &mut None);
      }
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
          .compare_exchange(n, n + 1, Ordering::Relaxed, Ordering::Relaxed)
      {
        n = e;
        continue;
      }
      return Some(Arc { weak: self.clone() });
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
    if self.data().alloc_ref_count.fetch_sub(1, Ordering::Relaxed) == 1 {
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
