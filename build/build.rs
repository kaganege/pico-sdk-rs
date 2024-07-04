use config::*;
use flate2::read::GzDecoder;
use std::{
  env,
  ffi::OsStr,
  fs,
  io::{Read, Seek, Write},
  path::{Path, PathBuf},
  str,
};

#[cfg_attr(target_os = "windows", path = "config/windows.rs")]
#[cfg_attr(target_os = "linux", path = "config/linux.rs")]
#[cfg_attr(target_os = "macos", path = "config/macos.rs")]
#[cfg_attr(
  not(any(target_os = "windows", target_os = "linux", target_os = "macos")),
  path = "config/default.rs"
)]
mod config;

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
struct BuildInfo {
  pub compile_definitions: Vec<String>,
  pub compile_options: Vec<String>,
  pub include_dirs: Vec<String>,
  pub link_flags: Vec<String>,
}

const BOARD_FEATURES: [&str; 2] = ["PICO", "PICO_W"];

fn main() {
  println!("cargo::rerun-if-changed=src/lib.rs");
  println!("cargo::rerun-if-changed=build/CMakeLists.txt");

  // By default, Cargo will re-run a build script whenever
  // any file in the project changes. By specifying `memory.x`
  // here, we ensure the build script is only re-run when
  // `memory.x` is changed.
  println!("cargo:rerun-if-changed=memory.x");

  let mut boards = BOARD_FEATURES
    .iter()
    .filter(|board| env::var(format!("CARGO_FEATURE_{board}")).is_ok());
  let board = boards.next().and_then(|board| Some(*board));
  assert!(
    boards.next().is_none(),
    "You can't specify more than one board."
  );

  let profile = env::var("PROFILE").unwrap();
  let project_dir = env::current_dir().unwrap();
  let current_dir = project_dir.join("build");
  let assets = current_dir.join("assets");
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  let sdk_lib_dir = out_dir.join("sdk");
  let mut toolchain_path = out_dir.join("toolchain");
  let ninja_path = out_dir.join("ninja");

  if !assets.exists() {
    fs::create_dir(&assets).expect("An error occurred while creating assets folder");
  }

  if TOOLCHAIN_DOWNLOAD_URL.is_empty() {
    panic!("The toolchain does not have any binaries for your operating system or arch.");
  } else if !toolchain_path.exists() {
    let toolchain_archive = Path::new(TOOLCHAIN_DOWNLOAD_URL).file_name().unwrap();
    let toolchain_archive_path = assets.join(toolchain_archive);
    let mut file_options = fs::OpenOptions::new();
    file_options.read(true);

    let mut file;
    if !toolchain_archive_path.exists() {
      file = file_options
        .write(true)
        .create(true)
        .open(&toolchain_archive_path)
        .expect("Couldn't open toolchain archive");
      reqwest::blocking::get(TOOLCHAIN_DOWNLOAD_URL)
        .expect("An error occurred while fetching the toolchain archive")
        .copy_to(&mut file)
        .unwrap();
    } else {
      file = file_options
        .open(&toolchain_archive_path)
        .expect("Couldn't open toolchain archive");
    }

    extract_archive(
      toolchain_archive_path.extension().unwrap(),
      &mut file,
      &toolchain_path,
    )
    .expect("Couldn't extract toolchain!");
  }

  if let Ok(mut dir) = toolchain_path.read_dir() {
    toolchain_path = dir.next().unwrap().unwrap().path();
  }

  if NINJA_DOWNLOAD_URL.is_empty() {
    panic!("Ninja does not have any binaries for your operating system or arch.");
  } else if !ninja_path.exists() {
    let ninja_archive_name = Path::new(NINJA_DOWNLOAD_URL).file_name().unwrap();
    let ninja_archive_path = assets.join(ninja_archive_name);
    let mut ninja_archive = fs::OpenOptions::new()
      .write(true)
      .read(true)
      .create(true)
      .open(&ninja_archive_path)
      .expect("Couldn't open ninja archive!");

    reqwest::blocking::get(NINJA_DOWNLOAD_URL)
      .expect("An error occurred while fetching ninja archive!")
      .copy_to(&mut ninja_archive)
      .unwrap();

    extract_archive(
      ninja_archive_path.extension().unwrap(),
      &mut ninja_archive,
      &ninja_path,
    )
    .expect("Couldn't extract toolchain!");
  }

  #[cfg(target_os = "windows")]
  {
    use std::io::Cursor;

    const TOOLS: [&str; 4] = [
      "pico-sdk-tools-config.cmake",
      "pico-sdk-tools-config-version.cmake",
      "pioasm.exe",
      "elf2uf2.exe",
    ];
    let is_tools_exists = TOOLS.iter().all(|tool| assets.join(tool).exists());

    if !is_tools_exists {
      let request = reqwest::blocking::get(PICO_SDK_TOOLS_DOWNLOAD_URL)
        .expect("An error occurred while fetching pico sdk tools!");
      let mut cursor = Cursor::new(request.bytes().unwrap());

      // Extract from memory
      extract_archive("zip", &mut cursor, &assets).expect("Couldn't extract toolchain!");
    }
  }

  let mut cmake_config = cmake::Config::new(&current_dir);

  // Ninja
  cmake_config.define("CMAKE_MAKE_PROGRAM", ninja_path.join("ninja.exe"));

  // Compiler
  // cmake_config.define(
  //   "CMAKE_C_COMPILER",
  //   toolchain_path.join("bin").join("arm-none-eabi-gcc.exe"),
  // );
  // cmake_config.define(
  //   "CMAKE_CXX_COMPILER",
  //   toolchain_path.join("bin").join("arm-none-eabi-g++.exe"),
  // );

  cmake_config.define("PICO_COMPILER", "pico_arm_gcc");

  if env::var("CARGO_FEATURE_EXTRAS").is_ok() {
    cmake_config.define(
      "PICO_EXTRAS_PATH",
      current_dir.join("..").join("pico-extras"),
    );
  }

  // Toolchain
  // cmake_config.define("PICO_TOOLCHAIN_PATH", toolchain_path);

  // Board
  if let Some(board) = board {
    cmake_config.define("PICO_BOARD", board.to_lowercase());
  }

  // Standart IO
  if env::var("CARGO_FEATURE_ENABLE_STDIO_USB").is_ok() {
    cmake_config.define("ENABLE_STDIO_USB", "");
  }

  if env::var("CARGO_FEATURE_ENABLE_STDIO_UART").is_ok() {
    cmake_config.define("ENABLE_STDIO_UART", "");
  }

  if env::var("CARGO_FEATURE_ENABLE_STDIO_SEMIHOSTING").is_ok() {
    cmake_config.define("ENABLE_STDIO_SEMIHOSTING", "");
  }

  // Output
  cmake_config.define(
    "CMAKE_ARCHIVE_OUTPUT_DIRECTORY",
    sdk_lib_dir.display().to_string(),
  );
  cmake_config.out_dir(sdk_lib_dir);

  let dst = cmake_config
    .generator("Ninja")
    .no_build_target(true)
    .build();
  let sdk_build_directory = dst.join("build");

  println!("cargo:rustc-link-search=native={}", dst.display());
  println!("cargo:rustc-link-lib=static=pico-sdk");

  let raw_build_info = fs::read(sdk_build_directory.join("build_info.toml"))
    .expect("An error occurred while reading build_info.toml");
  let build_info: BuildInfo =
    toml::from_str(str::from_utf8(&raw_build_info).expect("Invalid bytes in build_info.toml"))
      .expect("An error occurred while parsing build_info.toml");

  let bindings = bindgen::Builder::default()
    .raw_line("#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]")
    .use_core()
    .header(current_dir.join("pico-sdk.h").display().to_string())
    .header_contents(
      "_cyw43",
      if board.is_some_and(|b| b == "PICO_W") {
        "#include \"pico/cyw43_arch.h\""
      } else {
        ""
      },
    )
    .generate_comments(true)
    .generate_inline_functions(true)
    .disable_untagged_union()
    .prepend_enum_name(false)
    .layout_tests(false)
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .clang_arg(format!("-I{}", current_dir.join("lwipopts.h").display()))
    .clang_arg(format!("-I{}", toolchain_path.join("include").display()))
    .clang_arg(format!(
      "-I{}",
      toolchain_path
        .join("arm-none-eabi")
        .join("include")
        .display()
    ))
    .clang_args(build_info.include_dirs.iter().map(|dir| format!("-I{dir}")))
    .clang_args(build_info.compile_definitions)
    .generate()
    .expect("Unable to generate bindings");

  bindings
    .write_to_file(project_dir.join("src").join("pico_sdk.rs"))
    .expect("Couldn't write bindings!");

  // Put `memory.x` in our output directory and ensure it's
  // on the linker search path.
  fs::File::create(out_dir.join("memory.x"))
    .unwrap()
    .write_all(include_bytes!("memory.x"))
    .unwrap();
  println!("cargo:rustc-link-search={}", out_dir.display());

  // `--nmagic` is required if memory section addresses are not aligned to 0x10000,
  // for example the FLASH and RAM sections in your `memory.x`.
  // See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
  println!("cargo:rustc-link-arg=--nmagic");

  // Set the linker script to the one provided by cortex-m-rt.
  println!("cargo:rustc-link-arg=-Tlink.x");

  // for link_flag in build_info.link_flags {
  //   println!("cargo:rustc-link-arg={link_flag}");
  // }

  match profile.as_str() {
    "release" => {
      // Remove assets on release build to reduce size
      fs::remove_dir_all(assets).unwrap();
    }

    _ => (),
  }
}

fn extract_archive<E, F, O>(
  extension: E,
  archive_file: &mut F,
  output_path: O,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
  E: AsRef<OsStr>,
  F: Read + Seek,
  O: AsRef<Path>,
{
  match extension.as_ref().to_str() {
    Some("gz") => {
      let tar = GzDecoder::new(archive_file);
      let mut archive = tar::Archive::new(tar);
      archive.unpack(output_path)?;
    }
    Some("zip") => {
      let mut archive = zip::ZipArchive::new(archive_file)?;
      archive.extract(output_path)?;
    }
    _ => panic!("Unsupported toolchain archive!"),
  }

  Ok(())
}
