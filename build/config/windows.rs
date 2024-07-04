#[macro_use]
mod default;

#[cfg(target_arch = "x86_64")]
define_download_url! {
  TOOLCHAIN = "https://armkeil.blob.core.windows.net/developer/Files/downloads/gnu/13.2.rel1/binrel/arm-gnu-toolchain-13.2.rel1-mingw-w64-i686-arm-none-eabi.zip";
  NINJA = "https://github.com/ninja-build/ninja/releases/download/v1.12.1/ninja-win.zip";
  PICO_SDK_TOOLS = "https://github.com/will-v-pi/pico-sdk-tools/releases/download/v1.5.1-alpha-1/pico-sdk-tools-1.5.1-x64-win.zip"
}

#[cfg(not(target_arch = "x86_64"))]
define_download_url!(PICO_SDK_TOOLS);

#[cfg(not(target_arch = "x86_64"))]
pub use default::*;
