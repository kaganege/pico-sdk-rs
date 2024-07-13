#[macro_use]
mod default;

#[cfg(target_arch = "x86_64")]
define_download_url! {
  TOOLCHAIN = "https://developer.arm.com/-/media/Files/downloads/gnu/13.2.rel1/binrel/arm-gnu-toolchain-13.2.rel1-darwin-x86_64-arm-none-eabi.tar.xz";
  NINJA = "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-mac.zip"
}

#[cfg(all(target_arch = "arm", target_pointer_width = "64"))]
define_download_url!(TOOLCHAIN = "https://developer.arm.com/-/media/Files/downloads/gnu/13.2.rel1/binrel/arm-gnu-toolchain-13.2.rel1-darwin-arm64-arm-none-eabi.tar.xz");

#[cfg(not(all(
  any(target_arch = "arm", target_arch = "x86_64"),
  target_pointer_width = "64"
)))]
pub use default::*;
