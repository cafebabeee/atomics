#[cfg(test)]
pub mod test {
  use std::collections::VecDeque;
  use std::sync::Mutex;
  use std::time::Duration;
  use std::{sync::Arc, thread};
  #[test]
  fn test_add() {
    assert_eq!(2 + 2, 4);
  }

  #[test]
  fn test_spawn() {
    println!("Hello from a thread!, {:?}", thread::current().id());

    thread::spawn(|| {
      println!("Hello from a thread!, {:?}", thread::current().id());
    })
    .join()
    .unwrap();

    thread::scope(|t| {
      t.spawn(|| {
        println!("Hello from a thread!, {:?}", thread::current().id());
      });
      t.spawn(|| {
        println!("Hello from another thread!, {:?}", thread::current().id());
      });
    })
  }

  #[test]
  pub fn test_shared() {
    const X: [i8; 3] = [1, 2, 3];
    thread::scope(|t| {
      t.spawn(|| dbg!(&X));
      t.spawn(|| dbg!(&X));
    });

    let y = Box::leak(Box::new([4, 5, 6]));
    thread::scope(|t| {
      t.spawn(|| dbg!(&y));
      t.spawn(|| dbg!(&y));
    });

    let a = Arc::new([7, 8, 9]);
    thread::scope(|t| {
      let b = a.clone();
      t.spawn(move || dbg!(a));
      t.spawn(move || dbg!(b));
    });
  }

  #[test]
  pub fn test_mutex() {
    let n = Mutex::new(0);
    thread::scope(|s| {
      for _ in 0..10 {
        s.spawn(|| {
          {
            let mut guard = n.lock().unwrap();
            for _ in 0..100 {
              *guard += 1;
            }
          } // specific scope, equal to call drop(guard)
          // drop(guard); // drop guard before sleeping, it's will release the lock
          thread::sleep(Duration::from_secs(1)); // New!
        });
      }
    });
    assert_eq!(n.into_inner().unwrap(), 1000);
  }

  #[test]
  pub fn test_parking() {
    let queue = Mutex::new(VecDeque::new());

    thread::scope(|s| {
      // Consuming thread
      let t = s.spawn(|| {
        loop {
          let item = queue.lock().unwrap().pop_front();
          if let Some(item) = item {
            dbg!(item);
          } else {
            thread::park();
          }
        }
      });

      // Producing thread
      for i in 0.. {
        queue.lock().unwrap().push_back(i);
        t.thread().unpark();
        thread::sleep(Duration::from_secs(1));
      }
    });
  }
}
