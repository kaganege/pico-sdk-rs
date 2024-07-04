//! https://github.com/raspberrypi/pico-examples/blob/master/adc/onboard_temperature/onboard_temperature.c

#![no_std]
#![no_main]

use pico_sdk_sys::println;
use pico_sdk_sys::{
  adc_init, adc_read, adc_select_input, adc_set_temp_sensor_enabled, sleep_ms, stdio_init_all,
};

fn main() {
  unsafe {
    stdio_init_all();

    // Initialize hardware AD converter, enable onboard temperature sensor and
    // select its channel (do this once for efficiency, but beware that this is
    // a global operation).
    adc_init();
    adc_set_temp_sensor_enabled(true);
    adc_select_input(4);

    loop {
      let temperature = read_onboard_temperature();
      println!("Onboard temperature = {temperature:.2}");

      sleep_ms(990);
    }
  }
}

// References for this implementation:
// raspberry-pi-pico-c-sdk.pdf, Section '4.1.1. hardware_adc'
fn read_onboard_temperature() -> f64 {
  const CONVERSION_FACTOR: f64 = 3.3 / (1 << 12);

  let adc: f64 = adc_read() * CONVERSION_FACTOR;

  27.0 - (adc - 0.706) / 0.001721
}

#[panic_handler]
fn handler(_: &core::panic::PanicInfo) -> ! {
  cortex_m::interrupt::disable();

  loop {}
}
