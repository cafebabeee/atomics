#![allow(warnings)]
use std::thread;
use std::time::Duration;
mod arc;
mod atomics;
mod basic;
mod channel;
mod lock;
mod mem_order;
mod pin;
mod primitive;

use crate::lock::condvar::*;
use crate::lock::mutex::*;

fn main() {}
