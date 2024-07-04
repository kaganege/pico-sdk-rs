//! https://github.com/raspberrypi/pico-examples/blob/master/pico_w/wifi/blink/picow_blink.c

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use pico_sdk_sys::CYW43_WL_GPIO_LED_PIN;
use pico_sdk_sys::{cyw43_arch_gpio_put, cyw43_arch_init, sleep_ms};

use panic_halt as _;

#[entry]
fn main() -> ! {
  unsafe {
    if cyw43_arch_init() != 0 {
      panic!("Wi-Fi init failed");
    }

    loop {
      cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, true);
      sleep_ms(250);
      cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, false);
      sleep_ms(250);
    }
  }
}

// #[panic_handler]
// fn handler(_: &core::panic::PanicInfo) -> ! {
//   cortex_m::interrupt::disable();

//   loop {}
// }
