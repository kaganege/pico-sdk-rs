//! https://github.com/raspberrypi/pico-examples/blob/master/hello_world/usb/hello_usb.c

#![no_std]
#![no_main]

use pico_sdk_sys::println;
use pico_sdk_sys::{sleep_ms, stdio_init_all};

fn main() {
  unsafe {
    stdio_init_all();

    loop {
      println!("Hello, world!");
      sleep_ms(1000);
    }
  }
}

#[panic_handler]
fn handler(_: &core::panic::PanicInfo) -> ! {
  cortex_m::interrupt::disable();

  loop {}
}
