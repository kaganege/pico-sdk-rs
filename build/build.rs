use config::*;
use flate2::read::GzDecoder;
use git2::Repository;
use std::{
  env,
  ffi::OsStr,
  fs,
  io::{Read, Seek},
  path::{Path, PathBuf},
  str,
};
use tar::Archive as TarArchive;
use which::which;
use xz::read::XzDecoder;
use zip::ZipArchive;

#[cfg_attr(target_os = "windows", path = "config/windows.rs")]
#[cfg_attr(target_os = "linux", path = "config/linux.rs")]
#[cfg_attr(target_os = "macos", path = "config/macos.rs")]
#[cfg_attr(
  not(any(target_os = "windows", target_os = "linux", target_os = "macos")),
  path = "config/default.rs"
)]
mod config;

const PICO_SDK_URL: &str = "https://github.com/raspberrypi/pico-sdk";
const PICO_EXTRAS_URL: &str = "https://github.com/raspberrypi/pico-extras";

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
struct BuildInfo {
  pub compile_definitions: Vec<String>,
  pub compile_options: Vec<String>,
  pub include_dirs: Vec<String>,
  pub link_flags: Vec<String>,
}

fn main() {
  println!("cargo::rerun-if-changed=build/CMakeLists.txt");

  // We use prebuilt binaries on Windows
  #[cfg(not(target_os = "windows"))]
  assert!(
    check_for_installation_requirements(),
    "A native C/C++ compiler (clang or gcc) needs to be installed and in PATH for manual compilation of the tools."
  );

  let board = match env::var("CARGO_FEATURE_PICO_W") {
    Ok(_) => "pico_w",
    Err(_) => "pico",
  };
  let extras = env::var("CARGO_FEATURE_EXTRAS").is_ok();

  let project_dir = env::current_dir().unwrap();
  let current_dir = project_dir.join("build");
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  let assets = out_dir.join("assets");
  let build_dir = out_dir.join("pico-sdk");
  let sdk_dir = env_path_var_or("PICO_SDK_PATH", &build_dir);
  let extras_dir = env_path_var_or("PICO_EXTRAS_PATH", out_dir.join("pico-extras"));
  let ninja_path = which("ninja").unwrap_or(out_dir.join("ninja.exe"));
  let mut toolchain_path = env_path_var_or("PICO_TOOLCHAIN_PATH", out_dir.join("toolchain"));

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

  {
    const REQUIRED_FOLDERS: [&str; 4] = ["arm-none-eabi", "bin", "include", "lib"];
    let mut dir: Vec<fs::DirEntry> = toolchain_path
      .read_dir()
      .unwrap()
      .filter_map(Result::ok)
      .collect();

    if dir.len() == 1 {
      toolchain_path = dir.first().unwrap().path();
      dir = toolchain_path
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    }

    assert!(
      REQUIRED_FOLDERS.iter().all(|&folder| {
        dir
          .iter()
          .find(|d| d.file_name().to_str().is_some_and(|name| name == folder))
          .is_some()
      }),
      "Wrong toolchain path!"
    );
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
      &ninja_path.join(".."),
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

  if !sdk_dir.exists() {
    Repository::clone_recurse(PICO_SDK_URL, &sdk_dir)
      .expect("An error occurred while downloading pico sdk!");
  }

  if extras && !extras_dir.exists() {
    Repository::clone_recurse(PICO_EXTRAS_URL, &extras_dir)
      .expect("An error occurred while downloading pico extras!");
  }

  let mut cmake_config = cmake::Config::new(&current_dir);

  // Ninja
  cmake_config.define("CMAKE_MAKE_PROGRAM", ninja_path);

  // Compiler
  cmake_config.define(
    "CMAKE_C_COMPILER",
    toolchain_path.join("bin").join("arm-none-eabi-gcc.exe"),
  );
  cmake_config.define(
    "CMAKE_CXX_COMPILER",
    toolchain_path.join("bin").join("arm-none-eabi-g++.exe"),
  );

  cmake_config.define("PICO_COMPILER", "pico_arm_gcc");
  cmake_config.define("PICO_SDK_PATH", &sdk_dir);

  if extras {
    cmake_config.define("PICO_EXTRAS_PATH", extras_dir);
  }

  // Assets
  cmake_config.define("PICO_ASSETS_PATH", &assets);

  // Toolchain
  cmake_config.define("PICO_TOOLCHAIN_PATH", &toolchain_path);

  // Board
  cmake_config.define("PICO_BOARD", board);

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
  cmake_config.out_dir(build_dir);

  let dst = cmake_config
    .generator("Ninja")
    .no_build_target(true)
    .build();
  let sdk_build_dir = dst.join("build");

  println!("cargo:rustc-link-search=native={}", sdk_build_dir.display());
  println!("cargo:rustc-link-lib=static=pico-sdk");

  let raw_build_info = fs::read(sdk_build_dir.join("build_info.toml"))
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
      if board == "pico_w" {
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

  for link_flag in build_info.link_flags {
    println!("cargo:rustc-link-arg={link_flag}");
  }
}

fn env_path_var_or<K: AsRef<OsStr>, P: AsRef<Path>>(key: K, default: P) -> PathBuf {
  env::var(key)
    .and_then(|value| Ok(PathBuf::from(value)))
    .unwrap_or(default.as_ref().to_path_buf())
}

fn extract_archive<E, F, P>(
  extension: E,
  archive_file: &mut F,
  output_path: P,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
  E: AsRef<OsStr>,
  F: Read + Seek,
  P: AsRef<Path>,
{
  match extension.as_ref().to_str() {
    Some("gz") => {
      let tar = GzDecoder::new(archive_file);
      let mut archive = tar::Archive::new(tar);
      archive.unpack(output_path)?;
    }
    Some("xz") => {
      let tar = XzDecoder::new(archive_file);
      let mut archive = TarArchive::new(tar);
      archive.unpack(output_path)?;
    }
    Some("zip") => {
      let mut archive = ZipArchive::new(archive_file)?;
      archive.extract(output_path)?;
    }
    _ => return Err("Unsupported toolchain archive!".into()),
  }

  Ok(())
}

#[allow(unused)]
fn check_for_installation_requirements() -> bool {
  const SUPPORTED_COMPILERS: [&str; 3] = ["clang", "gcc", "cl"];

  SUPPORTED_COMPILERS
    .iter()
    .any(|compiler| which(compiler).is_ok())
}
