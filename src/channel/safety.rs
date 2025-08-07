use std::{
  cell::UnsafeCell,
  mem::MaybeUninit,
  sync::{Arc, atomic::AtomicBool, atomic::Ordering},
};

struct Channel<T> {
  // no longer `pub`
  message: UnsafeCell<MaybeUninit<T>>,
  ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

pub struct Sender<T> {
  chan: Arc<Channel<T>>,
}

pub struct Receiver<T> {
  chan: Arc<Channel<T>>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
  let chan = Arc::new(Channel {
    message: UnsafeCell::new(MaybeUninit::uninit()),
    ready: AtomicBool::new(false),
  });
  (
    Sender { chan: chan.clone() },
    Receiver { chan: chan.clone() },
  )
}

impl<T> Sender<T> {
  pub fn send(self, msg: T) {
    unsafe {
      (*self.chan.message.get()).write(msg);
    }
    self.chan.ready.store(true, Ordering::Release);
  }
}

impl<T> Receiver<T> {
  pub fn is_ready(&self) -> bool {
    self.chan.ready.load(Ordering::Relaxed)
  }

  pub fn receive(self) -> T {
    if !self.chan.ready.swap(false, Ordering::Acquire) {
      panic!("no message available!");
    }
    unsafe { (*self.chan.message.get()).assume_init_read() }
  }
}

impl<T> Drop for Channel<T> {
  fn drop(&mut self) {
    if (*self.ready.get_mut()) {
      unsafe {
        (*self.message.get_mut()).assume_init_drop();
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::channel;
  use std::thread;
  #[test]
  fn test_channel() {
    thread::scope(|s| {
      let (sender, receiver) = channel();
      let t = thread::current();
      s.spawn(move || {
        sender.send("hello world!");
        t.unpark();
      });
      while !receiver.is_ready() {
        thread::park();
      }
      dbg!(receiver.receive());
    });
  }
}
