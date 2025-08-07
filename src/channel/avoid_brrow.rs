use std::{
  cell::UnsafeCell,
  iter::Rev,
  marker::PhantomData,
  mem::MaybeUninit,
  sync::atomic::{AtomicBool, Ordering},
  thread,
};

pub struct Channel<T> {
  message: UnsafeCell<MaybeUninit<T>>,
  ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
  pub const fn new() -> Self {
    Self {
      message: UnsafeCell::new(MaybeUninit::uninit()),
      ready: AtomicBool::new(false),
    }
  }

  pub fn split(&mut self) -> (Sender<T>, Receiver<T>) {
    *self = Self::new();
    (
      Sender {
        chan: self,
        t: thread::current(),
      },
      Receiver {
        chan: self,
        _marker: PhantomData,
      },
    )
  }
}

impl<T> Drop for Channel<T> {
  fn drop(&mut self) {
    if *self.ready.get_mut() {
      unsafe { (*self.message.get_mut()).assume_init_drop() };
    }
  }
}

pub struct Sender<'a, T> {
  chan: &'a Channel<T>,
  t: thread::Thread,
}

impl<T> Sender<'_, T> {
  pub fn send(self, message: T) {
    unsafe { (*self.chan.message.get()).write(message) };
    self.chan.ready.store(true, Ordering::Release);
    self.t.unpark(); // New!
  }
}

pub struct Receiver<'a, T> {
  chan: &'a Channel<T>,
  _marker: PhantomData<*const ()>,
}

impl<T> Receiver<'_, T> {
  pub fn receive(self) -> T {
    while !self.chan.ready.swap(false, Ordering::Acquire) {
      thread::park();
    }
    unsafe { (*self.chan.message.get()).assume_init_read() }
  }
}

#[cfg(test)]
mod tests {

  use super::Channel;
  use std::{thread, time::Duration};

  #[test]
  fn test_channel() {
    let mut chan = Channel::new();
    thread::scope(|s| {
      let (sender, recevier) = chan.split();
      s.spawn(|| {
        sender.send("hello!");
        thread::sleep(Duration::from_millis(rand::random::<u8>() as u64));
      });
      dbg!(recevier.receive());
    });
    thread::scope(|s| {
      let (sender, recevier) = chan.split();
      s.spawn(|| {
        sender.send("world!");
        thread::sleep(Duration::from_millis(rand::random::<u8>() as u64));
      });
      dbg!(recevier.receive());
    });
  }
}
