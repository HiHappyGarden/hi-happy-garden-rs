# Generated Cmake Pico project file

cmake_minimum_required(VERSION 3.16...3.24)

if(NOT DEFINED PICO_SDK_PATH)
    set(PICO_SDK_PATH "${CMAKE_SOURCE_DIR}/pico-sdk" CACHE STRING "Path to the PICO SDK")
endif()
set(PICO_BOARD pico2_w)

set(CMAKE_C_STANDARD 17)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

message("Build type: ${CMAKE_BUILD_TYPE}")

# Initialise pico_sdk from installed location
# (note this can come from environment, CMake cache etc)

# == DO NOT EDIT THE FOLLOWING LINES for the Raspberry Pi Pico VS Code Extension to work ==
if(WIN32)
    set(USERHOME $ENV{USERPROFILE})
else()
    set(USERHOME $ENV{HOME})
endif()
set(sdkVersion 2.2.0)
set(toolchainVersion 14_2_Rel1)
set(picotoolVersion 2.2.0-a4)
set(picoVscode ${PICO_SDK_PATH}/cmake/pico-vscode.cmake)
if (EXISTS ${picoVscode})
    include(${picoVscode})
endif()
# ====================================================================================

# Pull in Raspberry Pi Pico SDK (must be before project)
include(${PICO_SDK_PATH}/external/pico_sdk_import.cmake)

# Set the FreeRTOS kernel path and config directory BEFORE adding the port
set(FREERTOS_KERNEL_PATH ${CMAKE_SOURCE_DIR}/FreeRTOS-Kernel)
set(FREERTOS_CONFIG_FILE_DIRECTORY ${CMAKE_SOURCE_DIR}/inc/hhg-config)

# Include the RP2350_ARM_NTZ port BEFORE pico_sdk_init()
add_subdirectory(${FREERTOS_KERNEL_PATH}/portable/ThirdParty/GCC/RP2350_ARM_NTZ/)

project(hi-happy-garden-rs VERSION "0.1.0" LANGUAGES CXX C ASM)

add_definitions(-DCYW43_LWIP=1)
add_definitions(-DCYW43_HOST_NAME=\"${PROJECT_NAME}\")

# Initialise the Raspberry Pi Pico SDK
pico_sdk_init()

message(STATUS "PROJECT_SOURCE_DIR=${PROJECT_SOURCE_DIR}")
message(STATUS "CMAKE_BINARY_DIR=${CMAKE_BINARY_DIR}")

# Include the secrets file if it exists
if(EXISTS "${CMAKE_SOURCE_DIR}/secrets.cmake")
    include(secrets.cmake)
    message(STATUS "Found secrets.cmake")
else()
    message(STATUS "Not found secrets.cmake")
endif()



# --- cJSON Integration from GitHub ---
include(FetchContent)

FetchContent_Declare(
    cJSON
    GIT_REPOSITORY https://github.com/DaveGamble/cJSON.git
    GIT_TAG        v1.7.19
    PATCH_COMMAND sed -i.bak "s/cmake_minimum_required(VERSION 3.0)/cmake_minimum_required(VERSION 3.5)/" CMakeLists.txt || sed -i "" "s/cmake_minimum_required(VERSION 3.0)/cmake_minimum_required(VERSION 3.5)/" CMakeLists.txt || true
)

# Configure cJSON options
set(ENABLE_CJSON_TEST OFF CACHE BOOL "Disable cJSON tests")
set(ENABLE_CJSON_UTILS ON CACHE BOOL "Enable cJSON utils")
set(BUILD_SHARED_AND_STATIC_LIBS OFF CACHE BOOL "Build only static library")
set(BUILD_SHARED_LIBS OFF CACHE BOOL "Build static library")

# Suppress deprecation warnings
set(CMAKE_WARN_DEPRECATED OFF CACHE BOOL "" FORCE)

FetchContent_MakeAvailable(cJSON)

# Re-enable deprecation warnings for the rest of the project
set(CMAKE_WARN_DEPRECATED ON CACHE BOOL "" FORCE)

# --- littlefs Integration from GitHub ---
FetchContent_Declare(
    littlefs
    GIT_REPOSITORY https://github.com/littlefs-project/littlefs.git
    GIT_TAG        v2.11.2
)

FetchContent_MakeAvailable(littlefs)

# Create a CMake target for littlefs since it doesn't have CMakeLists.txt
if(NOT TARGET littlefs)
    add_library(littlefs STATIC
        ${littlefs_SOURCE_DIR}/lfs.c
        ${littlefs_SOURCE_DIR}/lfs_util.c
    )
    
    target_compile_definitions(littlefs PUBLIC
        #LFS_THREADSAFE=1
        #LFS_NO_DEBUG=1
    )

    target_include_directories(littlefs PUBLIC
        ${littlefs_SOURCE_DIR}
    )
endif()

# Configure Rust build
set(CARGO_TARGET "thumbv8m.main-none-eabi")
set(CARGO_BUILD_DIR "${CMAKE_BINARY_DIR}/main")

# Default configuration options
option(HHG_DEFAULT_WIFI_HOSTNAME "Default WiFi hostname" "${PROJECT_NAME}")
option(HHG_DEFAULT_WIFI_AUTH "Enable WiFi authentication by default" 3)
option(HHG_DEFAULT_WIFI_ENABLED "Enable WiFi by default" OFF)
option(HHG_DEFAULT_TIMEZONE "Default timezone offset" 0)
option(HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED "Enable daylight saving time by default" OFF)
option(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH "DST start month (1-12)" 2)
option(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY "DST start day (1-31)" 31)
option(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR "DST start hour (0-23)" 2)
option(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH "DST end month (1-12)" 9)
option(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY "DST end day (1-31)" 31)
option(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR "DST end hour (0-23)" 3)
option(HHG_DEFAULT_NTP_MSG_LEN "Default NTP message length in bytes" 48)
option(HHG_DEFAULT_NTP_PORT "Default NTP server port" 123)
option(HHG_DEFAULT_NTP_SERVER "Default NTP server address" "0.europe.pool.ntp.org")
option(HHG_TESTS "Enable tests" OFF)

# AES encryption salt values for key derivation (can be customized per deployment)
set(HHG_AES_KEY_SALT "AES_KEY" CACHE STRING "Salt for AES key derivation from unique_id")
set(HHG_AES_IV_SALT "AES_IV" CACHE STRING "Salt for AES IV derivation from unique_id")

if(HHG_DEFAULT_WIFI_HOSTNAME)
    set(HHG_DEFAULT_WIFI_HOSTNAME "${HHG_DEFAULT_WIFI_HOSTNAME}")
else()
    set(HHG_DEFAULT_WIFI_HOSTNAME "${PROJECT_NAME}")
endif()

if(HHG_DEFAULT_TIMEZONE)
    set(HHG_DEFAULT_TIMEZONE "${HHG_DEFAULT_TIMEZONE}")
else()
    set(HHG_DEFAULT_TIMEZONE 60)
endif()


if(HHG_DEFAULT_WIFI_AUTH)
    set(HHG_DEFAULT_WIFI_AUTH ${HHG_DEFAULT_WIFI_AUTH})
else()
    set(HHG_DEFAULT_WIFI_AUTH 3)
endif()

if(HHG_DEFAULT_WIFI_ENABLED)
    set(RUST_WIFI_ENABLED "true")
else()
    set(RUST_WIFI_ENABLED "false")
endif()

if(HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED)
    set(RUST_DAYLIGHT_SAVING_ENABLED "true")
else()
    set(RUST_DAYLIGHT_SAVING_ENABLED "false")
endif()

if(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH)
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH})
else()
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH 2)
endif()

if(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY)
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY})
else()
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY 31)
endif()

if(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR)
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR})
else()
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR 2)
endif()

if(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH)
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH})
else()
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH 9)
endif()

if(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY)
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY})
else()
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY 31)
endif()

if(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR)
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR})
else()
    set(HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR 3)
endif()

if(HHG_DEFAULT_NTP_MSG_LEN)
    set(HHG_DEFAULT_NTP_MSG_LEN ${HHG_DEFAULT_NTP_MSG_LEN})
else()
    set(HHG_DEFAULT_NTP_MSG_LEN 48)
endif()

if(HHG_DEFAULT_NTP_PORT)
    set(HHG_DEFAULT_NTP_PORT ${HHG_DEFAULT_NTP_PORT})
else()
    set(HHG_DEFAULT_NTP_PORT 123)
endif()

if(HHG_DEFAULT_NTP_SERVER)
    set(HHG_DEFAULT_NTP_SERVER ${HHG_DEFAULT_NTP_SERVER})
else()
    set(HHG_DEFAULT_NTP_SERVER "0.europe.pool.ntp.org")
endif()

if(HHG_TESTS)
    set(HHG_TESTS_FEATURE "tests")
    add_compile_definitions(HHG_TESTS=1)
else()
    set(HHG_TESTS_FEATURE "")
endif()

message(STATUS "Building parameters --")
message(STATUS "HHG_DEFAULT_WIFI_SSID: ${HHG_DEFAULT_WIFI_SSID}")
if(HHG_DEFAULT_WIFI_PASSWORD) 
    message(STATUS "HHG_DEFAULT_WIFI_PASSWORD: is set")
endif()
message(STATUS "HHG_DEFAULT_WIFI_HOSTNAME: ${HHG_DEFAULT_WIFI_HOSTNAME}")
message(STATUS "HHG_DEFAULT_WIFI_AUTH: ${HHG_DEFAULT_WIFI_AUTH}")
message(STATUS "HHG_DEFAULT_WIFI_ENABLED: ${RUST_WIFI_ENABLED}")
message(STATUS "HHG_DEFAULT_TIMEZONE: ${HHG_DEFAULT_TIMEZONE}")
message(STATUS "HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED: ${RUST_DAYLIGHT_SAVING_ENABLED}")
message(STATUS "HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH: ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH}")
message(STATUS "HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY: ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY}")
message(STATUS "HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR: ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR}")
message(STATUS "HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH: ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH}")
message(STATUS "HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY: ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY}")
message(STATUS "HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR: ${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR}")
message(STATUS "HHG_DEFAULT_NTP_MSG_LEN: ${HHG_DEFAULT_NTP_MSG_LEN}")
message(STATUS "HHG_DEFAULT_NTP_PORT: ${HHG_DEFAULT_NTP_PORT}")
message(STATUS "HHG_DEFAULT_NTP_SERVER: ${HHG_DEFAULT_NTP_SERVER}")
message(STATUS "HHG_TESTS: ${HHG_TESTS}")
if(HHG_AES_KEY_SALT) 
    message(STATUS "HHG_AES_KEY_SALT: is set")
endif()
if(HHG_AES_IV_SALT) 
    message(STATUS "HHG_AES_IV_SALT: is set")
endif()

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/hhg-config.h.in ${CMAKE_CURRENT_SOURCE_DIR}/inc/hhg-config/pico/hhg-config.h)


# Determine Rust build profile based on CMAKE_BUILD_TYPE
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_PROFILE "")
    set(RUST_PROFILE_DIR "debug")
else()
    set(RUST_PROFILE "--release")
    set(RUST_PROFILE_DIR "release")
endif()

# Rust library paths
set(RUST_PATH ${CMAKE_SOURCE_DIR}/main/target/${CARGO_TARGET}/${RUST_PROFILE_DIR}/libmain.a)


# Track dependencies for at-parser-rs
file(GLOB_RECURSE HHG_PARSER_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_SOURCE_DIR}/at-parser-rs/src/*.rs"
    "${CMAKE_SOURCE_DIR}/at-parser-rs/Cargo.toml"
)

# Track dependencies for cjson-rs
file(GLOB_RECURSE PARSER_RS_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_SOURCE_DIR}/cjson-rs/src/*.rs"
    "${CMAKE_SOURCE_DIR}/cjson-rs/Cargo.toml"
)

# Track dependencies for osal-rs
file(GLOB_RECURSE OSAL_RS_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs/src/*.rs"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs-build/src/*.rs"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs-build/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs-serde/src/*.rs"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs-serde/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs-serde/derive/src/*.rs"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs-serde/derive/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/osal-rs/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/osal-rs/osal-rs/build.rs"
)

# Track dependencies for main
file(GLOB_RECURSE RUST_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_SOURCE_DIR}/main/src/*.rs"
    "${CMAKE_SOURCE_DIR}/main/Cargo.toml"
    "${CMAKE_SOURCE_DIR}/main/build.rs"
)

# --- Automatic compilation of main ---
set(FREERTOS_CONFIG_PATH "inc/hhg-config/pico/FreeRTOSConfig.h")
set(CARGO_BUILD_FLAGS --config "env.FREERTOS_CONFIG_PATH='${CMAKE_SOURCE_DIR}/${FREERTOS_CONFIG_PATH}'")

add_custom_command(
    OUTPUT ${RUST_PATH}
    COMMAND ${CMAKE_COMMAND} -E env 
        HHG_DEFAULT_WIFI_SSID="${HHG_DEFAULT_WIFI_SSID}"
        HHG_DEFAULT_WIFI_PASSWORD="${HHG_DEFAULT_WIFI_PASSWORD}"
        HHG_DEFAULT_WIFI_HOSTNAME="${HHG_DEFAULT_WIFI_HOSTNAME}"
        HHG_DEFAULT_WIFI_ENABLED="${RUST_WIFI_ENABLED}"
        HHG_DEFAULT_TIMEZONE="${HHG_DEFAULT_TIMEZONE}"
        HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED="${RUST_DAYLIGHT_SAVING_ENABLED}"
        HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH="${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH}"
        HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY="${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY}"
        HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR="${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR}"
        HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH="${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH}"
        HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY="${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY}"
        HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR="${HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR}"
        HHG_DEFAULT_NTP_MSG_LEN="${HHG_DEFAULT_NTP_MSG_LEN}"
        HHG_DEFAULT_NTP_PORT="${HHG_DEFAULT_NTP_PORT}"
        HHG_DEFAULT_NTP_SERVER="${HHG_DEFAULT_NTP_SERVER}"
        HHG_AES_KEY_SALT="${HHG_AES_KEY_SALT}"
        HHG_AES_IV_SALT="${HHG_AES_IV_SALT}"
        cargo build --target ${CARGO_TARGET} ${RUST_PROFILE} --features pico,${HHG_TESTS_FEATURE} 
    DEPENDS ${RUST_SOURCES} ${OSAL_RS_SOURCES} ${HHG_PARSER_SOURCES} ${PARSER_RS_SOURCES}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/main
    COMMENT "Building Rust library: main"
    VERBATIM
    USES_TERMINAL
)

add_custom_target(main ALL DEPENDS ${RUST_PATH})


file(GLOB_RECURSE INCLUDES CONFIGURE_DEPENDS "inc/*.hpp" "inc/*.h" "osal-rs/osal-rs-build/osal-rs-ffi-freertos/inc/*.h" "osal-rs/osal-rs-porting/freeretos/inc/*.h")
file(GLOB_RECURSE SOURCES CONFIGURE_DEPENDS "src/*.cpp" "src/*.c" "src/pico/*.cpp" "src/pico/*.c"  "osal-rs/osal-rs-build/osal-rs-ffi-freertos/src/*.c" "osal-rs/osal-rs-porting/freeretos/src/*.c")

add_executable(hi-happy-garden-rs 
        ${SOURCES}
        ${INCLUDES}
)

# Add dependency on Rust build
add_dependencies(${PROJECT_NAME} main)

pico_set_program_name(${PROJECT_NAME} "hi-happy-garden-rs")
pico_set_program_version(${PROJECT_NAME} ${PROJECT_VERSION})

# Modify the below lines to enable/disable output over UART/USB
pico_enable_stdio_uart(${PROJECT_NAME} 1)
pico_enable_stdio_usb(${PROJECT_NAME} 0)

# Add the standard include files to the build
target_include_directories(${PROJECT_NAME} PRIVATE
        ${CMAKE_CURRENT_LIST_DIR}
        ${CMAKE_CURRENT_LIST_DIR}/inc/hhg-config/pico
        ${CMAKE_CURRENT_LIST_DIR}/inc/hhg-ffi/pico
        ${CMAKE_CURRENT_LIST_DIR}/osal-rs/osal-rs-build/osal-rs-ffi-freertos/inc
        ${CMAKE_CURRENT_LIST_DIR}/osal-rs/osal-rs-porting/freeretos/inc
)

# Add any user requested libraries
target_link_libraries(${PROJECT_NAME} 
        ${RUST_PATH}
        pico_cyw43_arch_lwip_sys_freertos
        pico_lwip_mbedtls
        pico_mbedtls
        pico_lwip_mqtt
        pico_stdlib
        pico_multicore
        pico_unique_id
        hardware_gpio
        hardware_pwm
        hardware_adc
        hardware_i2c
        hardware_flash
        hardware_sha256
        hardware_powman
        FreeRTOS-Kernel
        FreeRTOS-Kernel-Heap4
        cjson
        littlefs
)

# Suppress stack warning
target_link_options(${PROJECT_NAME} PRIVATE -Wl,-z,noexecstack)

pico_add_extra_outputs(${PROJECT_NAME})

