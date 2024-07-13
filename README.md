# pico-sdk-rs

[Pico SDK](https://github.com/raspberrypi/pico-sdk) bindings for the Rust
programming language.

> [!WARNING]
> Most bindings are generated by
> [rust-bindgen](https://rust-lang.github.io/rust-bindgen). If any bindings are
> missing, [create an issue](https://github.com/kaganege/pico-sdk-sys/issues/new).

## Table of Contents

- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
- [Usage](#usage)
- [Features](#features)
- [Rust version requirements](#rust-version-requirements)
- [Version of Pico SDK](#version-of-pico-sdk)
- [License](#license)

## Getting Started

Since this crate depends on the Pico C SDK and other tools
([see the officialdocumentation](https://rptl.io/pico-c-sdk)), these must be
downloaded or compiled first. This crate will automatically download or compile
these tools for you, but it is also possible to manually download and the crate
will pick it up accordingly.

### Prerequisites

If the tools can already be found on your system, they will be used.

Otherwise, the following dependencies are needed to compile and build this crate:

- [Arm GNU Toolchain](https://developer.arm.com/Tools%20and%20Software/GNU%20Toolchain)
- [Ninja](https://ninja-build.org)

#### Environment variables

- `PICO_TOOLCHAIN_PATH` Path to Arm GNU Toolchain. It must contain
  `arm-none-eabi`, `bin`, `include`, `lib` folders.
- `PATH` This crate searches `ninja` in the PATH so be sure ninja is in the PATH.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
pico-sdk-sys = "0.1.0"
```

See the [official documentation](https://rptl.io/pico-c-sdk) for more details.
Examples can be found in the
[kaganege/pico-sdk-rs-examples](https://github.com/kaganege/pico-sdk-rs-examples).

## Features

By default `enable-stdio-usb` feature is enabled.

- `pico-w`: Enables WiFi support.
- `extras`: Adds [pico-extras](https://github.com/raspberrypi/pico-extras) bindings.
- `alloc`: Uses Arm GNU Toolchains allocators.
- `enable-stdio-uart`: Enables logging over UART.
- `enable-stdio-usb`: Enables logging over USB.
- `full`: Enables `extras` and `alloc` features.

## Rust version requirements

pico-sdk-rs works with stable Rust, and typically works with the most recent
prior stable release as well.

## Version of Pico SDK

Currently this library using [pico-sdk 1.5.1](https://github.com/raspberrypi/pico-sdk/releases/tag/1.5.1)
(or newer patch versions).

## License

Licensed under MIT license ([LICENSE](LICENSE) or
[opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
