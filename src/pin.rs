use std::{env::consts, marker::PhantomPinned, ops::Add, pin::Pin};

struct AddrTracker(Option<usize>);

impl AddrTracker {
  fn check_addr(&mut self) {
    let prev = self as *mut Self as usize;
    dbg!(prev);
    match self.0 {
      None => self.0 = Some(prev),
      Some(addr) => {
        dbg!(prev);
        dbg!(addr);
      }
    }
  }
}

fn move_it(mut tracker: AddrTracker) {
  println!("Moving tracker...");
  tracker.check_addr();
}

#[derive(Debug, Clone)]
struct SelfRef {
  name: String,
  // pointer to the self's name
  ptr: *const String,
}

#[derive(Debug)]
struct Unmovable {
  val: String,
  _marker: PhantomPinned,
}

struct PinSelf {
  name: Pin<Box<Unmovable>>,
  ptr: *const Unmovable,
}

impl PinSelf {
  fn new(name: String) -> Self {
    let pinned_name = Box::pin(Unmovable {
      val: name,
      _marker: PhantomPinned,
    });
    let ptr = pinned_name.as_ref().get_ref() as *const Unmovable;
    Self {
      name: pinned_name,
      ptr,
    }
  }
}

impl SelfRef {
  fn new(name: String) -> Self {
    let mut s = Self {
      name,
      ptr: std::ptr::null(),
    };
    s.ptr = &s.name as *const String;
    s
  }
}

fn r#move(mut self_ref: SelfRef) {
  println!("Moving SelfRef...");
  dbg!(self_ref.name.as_ptr(), self_ref.ptr);
}

fn move_pin(mut pinned: PinSelf) {
  println!("Moving Pinned PinSelf...");
  dbg!(
    pinned.name.as_ref().get_ref() as *const Unmovable,
    pinned.ptr
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_addr() {
    let mut tracker = AddrTracker(None);
    tracker.check_addr();
    move_it(tracker);
  }

  #[test]
  fn test_self_ref() {
    let mut self_ref = SelfRef::new("test".into());

    dbg!(self_ref.ptr);
    // [src\pin.rs:61:5] self_ref.ptr = 0x0000004bc3afec50

    r#move(self_ref);
    //[src\pin.rs:44:3] self_ref.name.as_ptr() = 0x00000214944d98e0
    //[src\pin.rs:44:3] self_ref.ptr = 0x0000004bc3afec50
  }

  #[test]
  fn test_pin() {
    let mut pinned = PinSelf::new("pinned".into());
    dbg!(
      pinned.name.as_ref().get_ref() as *const Unmovable,
      pinned.ptr
    );
    move_pin(pinned);
  }

  #[test]
  fn test_movable() {
    let mut pinned = Box::pin(Unmovable {
      val: "Pinned!".into(),
      _marker: PhantomPinned,
    });
    // *pinned = Unmovable {
    //   val: "try to move".into(),
    //   _marker: PhantomPinned,
    // };

    let pinned_ref = pinned.as_ref().get_ref();
    // the trait `Unpin` is not implemented for `std::marker::PhantomPinned`
    // let pinned_mut = pinned.as_mut().get_mut();
  }
}
