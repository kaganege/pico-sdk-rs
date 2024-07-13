#[macro_use]
mod default;

#[cfg(target_arch = "x86_64")]
define_download_url! {
  TOOLCHAIN = "https://developer.arm.com/-/media/Files/downloads/gnu/13.2.rel1/binrel/arm-gnu-toolchain-13.2.rel1-x86_64-arm-none-eabi.tar.xz";
  NINJA = "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-linux.zip";
}

#[cfg(target_arch = "aarch64")]
define_download_url! {
  TOOLCHAIN = "https://developer.arm.com/-/media/Files/downloads/gnu/13.2.rel1/binrel/arm-gnu-toolchain-13.2.rel1-aarch64-arm-none-eabi.tar.xz";
  NINJA = "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-linux-aarch64.zip";
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
pub use default::*;
