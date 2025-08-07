#![allow(warnings)]
use std::thread;
mod arc;
mod atomics;
mod basic;
mod channel;
mod lock;
mod mem_order;

fn main() {
  let numbers = vec![1, 2, 3];

  thread::scope(|s| {
    s.spawn(|| {
      println!("length: {}", numbers.len());
    });
    s.spawn(|| {
      for n in &numbers {
        println!("{n}");
      }
    });
  });

  use std::sync::Arc;

  let a = Arc::new([1, 2, 3]);
  let b = a.clone();
  thread::scope(|s| {
    s.spawn(move || dbg!(a));
    s.spawn(move || dbg!(b));
  });
}
