//! [Pico SDK](https://github.com/raspberrypi/pico-sdk) Rust bindings

// https://lorenz-ruprecht.at/docu/pico-sdk/1.4.0/html/index.html

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod gpio;
#[macro_use]
mod io;
#[doc(hidden)]
mod pico_sdk;

#[doc(hidden)]
#[cfg(feature = "alloc")]
mod allocator;

pub use gpio::*;
pub use io::put_str_raw;
pub use pico_sdk::*;
