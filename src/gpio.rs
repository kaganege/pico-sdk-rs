//! GPIO bindings

use crate::pico_sdk;
use pico_sdk::gpio_set_pulls;
use pico_sdk::sio_hw_t;

pub const SIO_PTR: *mut sio_hw_t = 0xd0000000u32 as _;

/// Set a number of GPIOs to output
pub unsafe fn gpio_set_dir_out_masked(mask: u32) {
  (*SIO_PTR).gpio_oe_set = mask;
}

/// Set a number of GPIOs to input
pub unsafe fn gpio_set_dir_in_masked(mask: u32) {
  (*SIO_PTR).gpio_oe_clr = mask;
}

/// Set a single GPIO direction
pub unsafe fn gpio_set_dir(gpio: u32, out: u32) {
  let mask = 1u32 << gpio;

  if out != 0 {
    gpio_set_dir_out_masked(mask);
  } else {
    gpio_set_dir_in_masked(mask)
  }
}

/// Check if a specific GPIO direction is OUT
pub unsafe fn gpio_is_dir_out(gpio: u32) -> u32 {
  (*SIO_PTR).gpio_oe & (1 << gpio)
}

/// Get a specific GPIO direction
///
/// Returns `1` for out, `0` for in
pub unsafe fn gpio_get_dir(gpio: u32) -> u32 {
  gpio_is_dir_out(gpio)
}

/// Drive high every GPIO appearing in mask
pub unsafe fn gpio_set_mask(mask: u32) {
  (*SIO_PTR).gpio_set = mask;
}

/// Drive low every GPIO appearing in mask
pub unsafe fn gpio_clr_mask(mask: u32) {
  (*SIO_PTR).gpio_clr = mask;
}

/// Drive a single GPIO high/low
pub unsafe fn gpio_put(gpio: u32, value: u32) {
  let mask = 1u32 << gpio;

  if value != 0 {
    gpio_set_mask(mask);
  } else {
    gpio_clr_mask(mask);
  }
}

/// Drive all pins simultaneously
pub unsafe fn gpio_put_all(value: u32) {
  (*SIO_PTR).gpio_out = value;
}

/// Determine whether a GPIO is currently driven high or low
///
/// This function returns the high/low output level most recently assigned to a
/// GPIO via [gpio_put](gpio_put) or similar. This is the value that is
/// presented outward to the IO muxing, *not* the input level back from the pad
/// (which can be read using [gpio_put](gpio_put)).
///
/// To avoid races, this function must not be used for read-modify-write
/// sequences when driving GPIOs -- instead functions like [gpio_put](gpio_put)
/// should be used to atomically update GPIOs. This accessor is intended for
/// debug use only.
pub unsafe fn gpio_get_out_level(gpio: u32) -> bool {
  ((*SIO_PTR).gpio_out & (1 << gpio)) > 0
}

/// Set specified GPIO to be pulled up.
pub unsafe fn gpio_pull_up(gpio: u32) {
  gpio_set_pulls(gpio, true, false)
}

/// Set specified GPIO to be pulled down.
pub unsafe fn gpio_pull_down(gpio: u32) {
  gpio_set_pulls(gpio, false, true)
}
