use std::{
  ops::Deref,
  ptr::NonNull,
  sync::atomic::{AtomicUsize, Ordering, fence},
};

struct ArcData<T> {
  data: T,
  ref_count: AtomicUsize,
}

pub struct Arc<T> {
  ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
  pub fn new(data: T) -> Self {
    Arc {
      ptr: NonNull::from(Box::leak(Box::new(ArcData {
        data,
        ref_count: AtomicUsize::new(1),
      }))),
    }
  }

  fn data(&self) -> &ArcData<T> {
    unsafe { self.ptr.as_ref() }
  }

  fn get_mut(arc: &mut Self) -> Option<&mut T> {
    if arc.data().ref_count.load(Ordering::Relaxed) == 1 {
      fence(Ordering::Acquire);
      Some(unsafe { &mut arc.ptr.as_mut().data })
    } else {
      None
    }
  }
}

impl<T> Deref for Arc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &(self.data().data)
  }
}

impl<T> Clone for Arc<T> {
  fn clone(&self) -> Self {
    // self.data().ref_count.fetch_add(1, Ordering::Relaxed);
    if self.data().ref_count.fetch_add(1, Ordering::Relaxed) >= usize::MAX / 2 {
      std::process::abort();
    }
    Self { ptr: self.ptr }
  }
}

impl<T> Drop for Arc<T> {
  fn drop(&mut self) {
    if self.data().ref_count.fetch_sub(1, Ordering::Release) == 1 {
      // Ensure the data is dropped only when the last reference is dropped
      std::sync::atomic::fence(Ordering::Acquire);
      unsafe {
        drop(Box::from_raw(self.ptr.as_ptr()));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
  };

  static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);
  struct DetectDrop;
  impl Drop for DetectDrop {
    fn drop(&mut self) {
      NUM_DROPS.fetch_add(1, Ordering::Relaxed);
    }
  }

  #[test]
  fn test_arc() {
    use super::Arc;
    let x = Arc::new(("hello", DetectDrop));

    let y = x.clone();

    let t = thread::spawn(move || dbg!(x.0));

    dbg!(y.0);

    t.join().unwrap();

    dbg!(NUM_DROPS.load(Ordering::Relaxed));

    drop(y);

    dbg!(NUM_DROPS.load(Ordering::Relaxed));
  }
}
