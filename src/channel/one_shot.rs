use std::{
  cell::UnsafeCell,
  mem::MaybeUninit,
  sync::{
    Condvar, Mutex,
    atomic::{AtomicBool, AtomicPtr, Ordering},
  },
};

#[derive(Debug, PartialEq, Eq)]
enum State {
  Empty,
  Ready,
  Writing,
  Reading,
}

pub struct Channel<T> {
  msg: UnsafeCell<MaybeUninit<T>>,
  state: AtomicPtr<State>,
}

unsafe impl<T: Send> Sync for Channel<T> {}

impl<T> Channel<T> {
  pub fn new() -> Self {
    Channel {
      msg: UnsafeCell::new(MaybeUninit::uninit()),
      state: AtomicPtr::new(unsafe { &State::Empty as *const State as *mut State }),
    }
  }

  pub fn send(&self, val: T) {
    unsafe {
      if *self.state.swap(
        unsafe { &State::Writing as *const State as *mut State },
        Ordering::Relaxed,
      ) != State::Empty
      {
        panic!("Channel is not ready to send a message");
      }
    }

    unsafe { (*self.msg.get()).write(val) };
    self.state.store(
      unsafe { &State::Ready as *const State as *mut State },
      Ordering::Release,
    );
  }

  pub fn is_ready(&self) -> bool {
    unsafe { *self.state.load(Ordering::Acquire) == State::Ready }
  }

  pub fn receive(&self) -> T {
    if self
      .state
      .compare_exchange(
        unsafe { &State::Ready as *const State as *mut State },
        unsafe { &State::Reading as *const State as *mut State },
        Ordering::Acquire,
        Ordering::Relaxed,
      )
      .is_err()
    {
      panic!("Channel is not ready to receive a message");
    }
    unsafe { (*self.msg.get()).assume_init_read() }
  }
}

impl<T> Drop for Channel<T> {
  fn drop(&mut self) {
    unsafe {
      if *self.state.load(Ordering::Acquire) != State::Empty {
        return;
      }
      (*self.msg.get()).assume_init_drop();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Channel;
  use std::thread;

  #[test]
  fn test_channel() {
    let chan = Channel::new();
    let t = thread::current();
    thread::scope(|s| {
      s.spawn(|| {
        chan.send("hello");
        // This would panic since we can only send one message
        // chan.send("world");
        t.unpark();
      });
      while !chan.is_ready() {
        thread::park();
      }
      dbg!(chan.receive());
    })
  }
}
