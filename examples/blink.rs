//! https://github.com/raspberrypi/pico-examples/blob/master/blink/blink.c

#![no_std]
#![no_main]

use pico_sdk_sys::GPIO_OUT;
use pico_sdk_sys::{gpio_init, gpio_put, gpio_set_dir, sleep_ms};

const LED_PIN: u32 = pico_sdk_sys::PICO_DEFAULT_LED_PIN;

fn main() {
  unsafe {
    gpio_init(LED_PIN);
    gpio_set_dir(LED_PIN, GPIO_OUT);

    loop {
      gpio_put(LED_PIN, 1);
      sleep_ms(250);
      gpio_put(LED_PIN, 0);
      sleep_ms(250);
    }
  }
}

#[panic_handler]
fn handler(_: &core::panic::PanicInfo) -> ! {
  cortex_m::interrupt::disable();

  loop {}
}
