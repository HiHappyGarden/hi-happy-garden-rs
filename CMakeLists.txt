# Generated Cmake Pico project file

cmake_minimum_required(VERSION 3.16...3.24)

if(NOT DEFINED PICO_SDK_PATH)
    set(PICO_SDK_PATH "${CMAKE_SOURCE_DIR}/pico-sdk" CACHE STRING "Path to the PICO SDK")
endif()
set(PICO_BOARD pico2_w)

set(CMAKE_C_STANDARD 17)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

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
add_subdirectory(${FREERTOS_KERNEL_PATH}/portable/ThirdParty/Community-Supported-Ports/GCC/RP2350_ARM_NTZ FreeRTOS-Kernel)

project(hi-happy-garden-rs C CXX ASM)

add_definitions(-DCYW43_LWIP=1)
add_definitions(-DCYW43_HOST_NAME=\"${PROJECT_NAME}\")

file(GLOB_RECURSE INCLUDES CONFIGURE_DEPENDS "inc/*.hpp" "inc/*.h")
file(GLOB_RECURSE SOURCES CONFIGURE_DEPENDS "src/*.cpp" "src/*.c")

# Initialise the Raspberry Pi Pico SDK
pico_sdk_init()

# Add osal-rs-ffi-freertos library AFTER pico_sdk_init()
add_subdirectory(osal-rs/osal-rs-ffi-freertos)

# --- cJSON Integration from GitHub ---
include(FetchContent)
FetchContent_Declare(
    cJSON
    GIT_REPOSITORY https://github.com/DaveGamble/cJSON.git
    GIT_TAG        v1.7.19
)

# Configure cJSON options
set(ENABLE_CJSON_TEST OFF CACHE BOOL "Disable cJSON tests")
set(ENABLE_CJSON_UTILS ON CACHE BOOL "Enable cJSON utils")
set(BUILD_SHARED_AND_STATIC_LIBS OFF CACHE BOOL "Build only static library")
set(BUILD_SHARED_LIBS OFF CACHE BOOL "Build static library")

FetchContent_MakeAvailable(cJSON)

# Configure Rust build
set(CARGO_TARGET "thumbv8m.main-none-eabi")
set(CARGO_BUILD_DIR "${CMAKE_BINARY_DIR}/rust")
# set(Rust_CARGO_TARGET "thumbv8m.main-none-eabi")
# set(Rust_CARGO_TARGET_ARCH "thumbv8m")

# Determine Rust build profile based on CMAKE_BUILD_TYPE
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_PROFILE "")
    set(RUST_PROFILE_DIR "debug")
else()
    set(RUST_PROFILE "--release")
    set(RUST_PROFILE_DIR "release")
endif()


# Percorsi delle librerie Rust
set(RUST_HHG_HARDWARE_PATH ${CMAKE_SOURCE_DIR}/hhg-hardware/target/${CARGO_TARGET}/${RUST_PROFILE_DIR}/libhhg_hardware.a)
set(RUST_HHG_APP_PATH ${CMAKE_SOURCE_DIR}/hhg-app/target/${CARGO_TARGET}/${RUST_PROFILE_DIR}/libhhg_app.a)

# --- Compilazione automatica di hhg-hardware ---
add_custom_command(
    OUTPUT ${RUST_HHG_HARDWARE_PATH}
    COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}/hhg-hardware cargo build --target ${CARGO_TARGET} ${RUST_PROFILE} --features pico
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/hhg-hardware
    COMMENT "Building Rust library: hhg-hardware"
    VERBATIM
)

add_custom_target(hhg_hardware ALL DEPENDS ${RUST_HHG_HARDWARE_PATH})

# --- Compilazione automatica di hhg-app ---
add_custom_command(
    OUTPUT ${RUST_HHG_APP_PATH}
    COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}/hhg-app cargo build --target ${CARGO_TARGET} ${RUST_PROFILE}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/hhg-app
    COMMENT "Building Rust library: hhg-app"
    VERBATIM
)

add_custom_target(hhg_app ALL DEPENDS ${RUST_HHG_APP_PATH})

add_executable(hi-happy-garden-rs 
        ${SOURCES}
        ${INCLUDES}
)

# Add dependency on Rust build
add_dependencies(${PROJECT_NAME}  hhg_hardware hhg_app)

# Link Rust static library
target_link_libraries(${PROJECT_NAME}
    ${RUST_HHG_HARDWARE_PATH} 
    ${RUST_HHG_APP_PATH}
)

pico_set_program_name(${PROJECT_NAME} "hi-happy-garden-rs")
pico_set_program_version(${PROJECT_NAME} "0.1.0")

# Modify the below lines to enable/disable output over UART/USB
pico_enable_stdio_uart(${PROJECT_NAME} 1)
pico_enable_stdio_usb(${PROJECT_NAME} 0)

# Add the standard include files to the build
target_include_directories(${PROJECT_NAME} PRIVATE
        ${CMAKE_CURRENT_LIST_DIR}
        ${CMAKE_CURRENT_LIST_DIR}/inc/hhg-config/pico
        ${CMAKE_CURRENT_LIST_DIR}/inc/hhg-ffi/pico
        ${CMAKE_CURRENT_LIST_DIR}/osal-rs/osal-rs-ffi-freertos/inc
)

# Add any user requested libraries
target_link_libraries(${PROJECT_NAME} 
        pico_cyw43_arch_lwip_sys_freertos
        pico_lwip_mbedtls
        pico_mbedtls
        pico_lwip_mqtt
        pico_stdlib
        pico_multicore
        pico_unique_id
        FreeRTOS-Kernel
        FreeRTOS-Kernel-Heap4
        cjson
        osal-rs-ffi-freertos
)

# Suppress stack warning
target_link_options(${PROJECT_NAME} PRIVATE -Wl,-z,noexecstack)

pico_add_extra_outputs(${PROJECT_NAME})

