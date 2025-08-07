use std::{
  ptr::addr_of_mut,
  sync::atomic::{
    AtomicBool, AtomicI32, AtomicPtr,
    Ordering::{Acquire, Relaxed, Release, SeqCst},
    fence,
  },
  thread,
  time::Duration,
};

use rand::random;

static X: AtomicI32 = AtomicI32::new(0);
static Y: AtomicI32 = AtomicI32::new(0);

pub fn a() {
  X.store(10, Relaxed);
  Y.store(20, Relaxed);
}

pub fn b() {
  let y = Y.load(Relaxed);
  let x = X.load(Relaxed);
  dbg!(x, y);
}
static mut DATA: String = String::new();
static LOCK: AtomicBool = AtomicBool::new(false);

pub fn lock() {
  if LOCK.compare_exchange(false, true, Acquire, Relaxed).is_ok() {
    unsafe { (*(&raw mut DATA)).push('!') };
    LOCK.store(false, Relaxed); // Release the lock
  }
}
#[derive(Debug, Default)]
struct Data();
pub fn get_data() -> &'static Data {
  static PTR: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());

  let mut p = PTR.load(Acquire);

  if p.is_null() {
    p = Box::into_raw(Box::new(Data::default())); // generate new data
    if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
      // Safety: p comes from Box::into_raw right above,
      // and wasn't shared with any other thread.
      drop(unsafe { Box::from_raw(p) });
      p = e;
    }
  }

  // Safety: p is not null and points to a properly initialized value.
  unsafe { &*p }
}

static A: AtomicBool = AtomicBool::new(false);
static B: AtomicBool = AtomicBool::new(false);

static mut S: String = String::new();

pub fn seq_cst() {
  let a = thread::spawn(|| {
    A.store(true, SeqCst);
    if !B.load(SeqCst) {
      unsafe { (*addr_of_mut!(S)).push('!') };
    }
  });

  let b = thread::spawn(|| {
    B.store(true, SeqCst);
    if !A.load(SeqCst) {
      unsafe { (*addr_of_mut!(S)).push('!') };
    }
  });

  a.join().unwrap();
  b.join().unwrap();
}

static mut DATA_ARR: [u64; 10] = [0; 10];

const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

pub fn order_fence() {
  for i in 0..10 {
    thread::spawn(move || {
      //thread::sleep(Duration::from_millis(rand::random::<u16>() as u64));
      let data = rand::random::<u64>();
      unsafe { DATA_ARR[i] = data };
      READY[i].store(true, Release);
    });
  }
  thread::sleep(Duration::from_millis(500));
  let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Relaxed));
  if ready.contains(&true) {
    fence(Acquire);
    for i in 0..10 {
      if ready[i] {
        dbg!(unsafe { DATA_ARR[i] });
      }
    }
  }
}
