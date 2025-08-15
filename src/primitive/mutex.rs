use std::cell::UnsafeCell;

// pub struct Mutex {
//   m: Box<UnsafeCell<libc::pthread_mutex_t>>,
// }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mutex() {
    unsafe {
      libc::abs(6);
    }
    // let m = Mutex::new();
    // m.lock();
    // m.unlock();
  }
}
