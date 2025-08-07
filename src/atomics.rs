pub mod eg;
#[cfg(test)]
mod test {
  use crate::atomics::eg;
  use std::thread;
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
}
