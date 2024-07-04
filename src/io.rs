pub unsafe fn put_str_raw(value: &str) {
  for char in value.chars() {
    crate::pico_sdk::putchar_raw(char as _);
  }
}

::custom_print::define_macros!({ cprint, cprintln, cdbg }, concat, |value: &str| {
  unsafe {
    $crate::io::put_str_raw(value);
  }
});

::custom_print::define_macros!({ ceprint, ceprintln }, concat, |value: &str| {
  #[cfg(feature = "alloc")]
  unsafe {
    $crate::io::put_str_raw(format!("error: {value}").as_str());
  }

  #[cfg(not(feature = "alloc"))]
  unsafe {
    $crate::io::put_str_raw(value);
  }
});

#[macro_export]
macro_rules! flush {
  () => {
    unsafe { $crate::stdio_flush() }
  };
}
#[macro_export]
macro_rules! print { ($($args:tt)*) => { cprint!($($args)*) } }
#[macro_export]
macro_rules! println { ($($args:tt)*) => { cprintln!($($args)*) } }
#[macro_export]
macro_rules! eprint { ($($args:tt)*) => { ceprint!($($args)*) } }
#[macro_export]
macro_rules! eprintln { ($($args:tt)*) => { ceprintln!($($args)*) } }
#[macro_export]
macro_rules! dbg { ($($args:tt)*) => { cdbg!($($args)*) } }
