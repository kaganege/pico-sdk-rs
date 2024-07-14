# THIS IS A SLIGHTLY HACKY EXAMPLE PROJECT DESIGNED TO HELP YOU TOWARDS INCLUDING THE PICO SDK
# CODE OUTSIDE OF THE CMAKE BUILD
#
# Note that the Pico SDK is designed to make it easy for users to build applications rather to build
# other runtimes for the Pico.
#
# There is no included "static library" of the Pico SDK because all of the Pico SDK Libraries are configurable
# on a per binary basis, and are always compiled from scratch per binary within the normal Pico SDK CMake build
#
# Note you would want to customize this to add or remove the dependencies you want (e.g. you mau not want
# to depend on pico_stdlib, but rather pick and mix some of its constituents to give you a finer grain of control
#
# Note that this example produces a build_info.txt in the output with some approximate (we're not trying to
# rewrite CMake here!) compile and link flags based on the setup of your pico-sdk library
#
# Finally note that even copying the build and link flags from build_info.txt and linking against the generated
# library does not get you all the way, as you still need a second stage boot at the start of the binary (0x000-0x100)

cmake_minimum_required(VERSION 3.12)

# Pull in PICO SDK (must be before project)
include(../pico-sdk/pico_sdk_init.cmake)

if(DEFINED PICO_EXTRAS_PATH)
  include(${PICO_EXTRAS_PATH}/external/pico_extras_import.cmake)
endif()

project(pico-sdk C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

pico_sdk_init()

if (PICO_ON_DEVICE)
  include(hacky_cmake_helper.cmake)
  include(toml.cmake)

  if(WIN32)
    set(pico-sdk-tools_DIR "${PICO_ASSETS_PATH}")

    include(${pico-sdk-tools_DIR}/pico-sdk-tools-config.cmake)
    include(${pico-sdk-tools_DIR}/pico-sdk-tools-config-version.cmake)
  endif()

  add_library(pico-sdk pico-sdk.c)

  target_compile_definitions(pico-sdk PUBLIC
    PICO_NO_BUILD_TYPE_FEATURE=1)

  if(DEFINED ENABLE_STDIO_USB)
    pico_enable_stdio_usb(pico-sdk 1)
  else()
    pico_enable_stdio_usb(pico-sdk 0)
  endif()

  if(DEFINED ENABLE_STDIO_UART)
    pico_enable_stdio_uart(pico-sdk 1)
  else()
    pico_enable_stdio_uart(pico-sdk 0)
  endif()

  if(DEFINED ENABLE_STDIO_SEMIHOSTING)
    pico_enable_stdio_semihosting(pico-sdk 1)
  else()
    pico_enable_stdio_semihosting(pico-sdk 0)
  endif()

  target_link_libraries(pico-sdk PUBLIC pico_stdlib)

  # our hacky_cmake_helper does not deal with generator expressions for libraries, so pick some (assuming we want the pico versions)
  # you might choose not to have these libraries (which are mostly redirecting AEABI functions to our faster (possible ROM) equivalents
  # especially if you are skipping SDK initialization (which is required for some of them)
  target_link_libraries(pico-sdk PUBLIC
    pico_bit_ops_pico
    pico_divider_hardware
    pico_float_pico
    pico_double_pico
    pico_mem_ops_pico
    pico_int64_ops_pico

    hardware_adc
    hardware_base
    hardware_claim
    hardware_clocks
    hardware_divider
    hardware_dma
    hardware_flash
    hardware_gpio
    hardware_i2c
    hardware_interp
    hardware_irq
    hardware_pio
    hardware_pll
    hardware_pwm
    hardware_resets
    hardware_rtc
    hardware_spi
    hardware_sync
    hardware_timer
    hardware_uart
    hardware_vreg
    hardware_watchdog
  )

  if(PICO_BOARD STREQUAL "pico_w")
    target_link_libraries(pico-sdk PUBLIC pico_cyw43_arch_none)
  endif()


  # Compile definitions
  gather_vars(COMPILE_DEFINITIONS INTERFACE_COMPILE_DEFINITIONS _touched_cd pico-sdk)
  list(REMOVE_DUPLICATES COMPILE_DEFINITIONS)
  list(PREPEND COMPILE_DEFINITIONS "") # need a -D at the beginning
  string(REPLACE ";" " -D" COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS}")

  # Compile options
  gather_vars(COMPILE_OPTIONS INTERFACE_COMPILE_OPTIONS _touched_cf pico-sdk)
  list(REMOVE_DUPLICATES COMPILE_OPTIONS)
  # string(REPLACE ";" " " COMPILE_OPTIONS "${COMPILE_OPTIONS}")
  # Note that the Pico SDK has some options specific to the language being
  # compiled. All SDK has already been compiled, so we'll just strip these and
  # you'll have to deal with these yourselves
  string(REGEX REPLACE "\\$<\\$<COMPILE_LANGUAGE:CXX>:.*>" "" COMPILE_OPTIONS ${COMPILE_OPTIONS})

  # Include dirs
  gather_vars(INCLUDE_DIRECTORIES INTERFACE_INCLUDE_DIRECTORIES _touched_id pico-sdk)
  list(REMOVE_DUPLICATES INCLUDE_DIRECTORIES)
  # string(REPLACE ";" " " INCLUDE_DIRECTORIES "${INCLUDE_DIRECTORIES}")

  # Link options
  gather_vars(LINK_FLAGS INTERFACE_LINK_OPTIONS _touched_l pico-sdk)
  # string(REPLACE ";" " " LINK_FLAGS "${LINK_FLAGS}")
  string(REPLACE "LINKER:" "-Wl," LINK_FLAGS "${LINK_FLAGS}")

  # Target properties were scoped to the target being built, lets scope them
  # to pico-sdk
  string(REPLACE "$<TARGET_PROPERTY:" "$<TARGET_PROPERTY:pico-sdk," COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS}")
  to_toml_array("${COMPILE_DEFINITIONS}" COMPILE_DEFINITIONS_ARRAY " ")
  string(REPLACE "$<TARGET_PROPERTY:" "$<TARGET_PROPERTY:pico-sdk," COMPILE_OPTIONS "${COMPILE_OPTIONS}")
  to_toml_array("${COMPILE_OPTIONS}" COMPILE_OPTIONS_ARRAY ";")
  string(REPLACE "$<TARGET_PROPERTY:" "$<TARGET_PROPERTY:pico-sdk," INCLUDE_DIRECTORIES "${INCLUDE_DIRECTORIES}")
  to_toml_array("${INCLUDE_DIRECTORIES}" INCLUDE_DIRECTORIES_ARRAY ";")
  string(REPLACE "$<TARGET_PROPERTY:" "$<TARGET_PROPERTY:pico-sdk," LINK_FLAGS "${LINK_FLAGS}")
  to_toml_array("${LINK_FLAGS}" LINK_FLAGS_ARRAY ";")

  # The only way to get CMake to do some generator expression evaluation is to
  # generate a file
  file(GENERATE OUTPUT build_info.toml CONTENT
"compile_definitions = ${COMPILE_DEFINITIONS_ARRAY}
compile_options = ${COMPILE_OPTIONS_ARRAY}
include_dirs = ${INCLUDE_DIRECTORIES_ARRAY}
link_flags = ${LINK_FLAGS_ARRAY}")

endif()
