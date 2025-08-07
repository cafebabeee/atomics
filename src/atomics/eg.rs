use rand::prelude::*;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};
use std::thread;
use std::time::Duration;

pub fn bools() {
  static STOP: AtomicBool = AtomicBool::new(false);
  let some_work = || {
    // Some work to be done in the background thread.
    println!("Doing some work...");
  };
  // Spawn a thread to do the work.
  let background_thread = thread::spawn(move || {
    while !STOP.load(Relaxed) {
      some_work();
    }
  });

  // Use the main thread to listen for user input.
  for line in std::io::stdin().lines() {
    match line.unwrap().as_str() {
      "help" => println!("commands: help, stop"),
      "stop" => break,
      cmd => println!("unknown command: {cmd:?}"),
    }
  }

  // Inform the background thread it needs to stop.
  STOP.store(true, Relaxed);

  // Wait until the background thread finishes.
  background_thread.join().unwrap();
}

pub fn process() {
  // let num_done = AtomicUsize::new(0);
  // thread::scope(|s| {
  //   s.spawn(|| {
  //     // Some work to be done in the background thread.
  //     for i in 0..100 {
  //       thread::sleep(Duration::from_millis(99));
  //       num_done.store(i + 1, Relaxed);
  //     }
  //   });
  //   loop {
  //     let done = num_done.load(Relaxed);
  //     if done == 100 {
  //       break;
  //     }
  //     println!("Done: {done} / 100");
  //     print!("\r\x1B[K");
  //     thread::sleep(Duration::from_millis(100));
  //   }
  // });
  let num_done = &AtomicUsize::new(0);

  thread::scope(|s| {
    // Four background threads to process all 100 items, 25 each.
    for t in 0..4 {
      s.spawn(move || {
        for i in 0..25 {
          thread::sleep(Duration::from_millis(t * 25 + i));
          num_done.fetch_add(1, Relaxed);
        }
      });
    }

    // The main thread shows status updates, every second.
    loop {
      let n = num_done.load(Relaxed);
      if n == 100 {
        break;
      }
      println!("Working.. {n}/100 done");
      thread::sleep(Duration::from_secs(1));
    }
  });

  println!("Done!");
}

pub fn sync() {
  let num_done = AtomicUsize::new(0);
  let c_thread = thread::current();
  thread::scope(|s| {
    s.spawn(|| {
      // Some work to be done in the background thread.
      for i in 0..100 {
        thread::sleep(Duration::from_millis(99));
        num_done.store(i + 1, Relaxed);
        c_thread.unpark()
      }
    });
    loop {
      let done = num_done.load(Relaxed);
      if done == 100 {
        break;
      }
      println!("Done: {done} / 100");
      thread::park_timeout(Duration::from_millis(1000));
    }
  });
}

fn get_x() -> u64 {
  static X: AtomicU64 = AtomicU64::new(0);
  let mut x = X.load(Relaxed);
  if x == 0 {
    x = 1;
    thread::sleep(Duration::from_secs(3));
    X.store(x, Relaxed);
  }
  x
}

pub fn lazy_init() {
  let x = get_x();
  let y = get_x();
  println!("x: {x}, y: {y}");
}

pub fn allocate_new_id() {
  static NEXT_ID: AtomicU64 = AtomicU64::new(0);
  let next = NEXT_ID.load(Relaxed);
  NEXT_ID
    .fetch_update(Relaxed, Relaxed, |id| id.checked_add(1))
    .expect("To many Ids!");
}
pub fn lazy_onetime_init() -> u64 {
  static KEY: AtomicU64 = AtomicU64::new(0);
  let mut x = KEY.load(Relaxed);
  if x == 0 {
    let new_key = rand::rng().random(); // generate new key randomly
    match KEY.compare_exchange(0, new_key, Relaxed, Relaxed) {
      Ok(_) => new_key,
      Err(k) => k,
    }
  } else {
    x
  }
}
