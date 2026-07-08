cmake_minimum_required(VERSION 3.20)
project(rust_ethernet_ip_cpp_example LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

get_filename_component(REPO_ROOT "${CMAKE_CURRENT_LIST_DIR}/../.." ABSOLUTE)

if(NOT DEFINED RUST_ETHERNET_IP_NATIVE_LIB)
    if(WIN32)
        set(RUST_ETHERNET_IP_NATIVE_LIB "${REPO_ROOT}/target/release/rust_ethernet_ip.dll")
    elseif(APPLE)
        set(RUST_ETHERNET_IP_NATIVE_LIB "${REPO_ROOT}/target/release/librust_ethernet_ip.dylib")
    else()
        set(RUST_ETHERNET_IP_NATIVE_LIB "${REPO_ROOT}/target/release/librust_ethernet_ip.so")
    endif()
endif()

if(NOT EXISTS "${RUST_ETHERNET_IP_NATIVE_LIB}")
    message(FATAL_ERROR "Native library not found: ${RUST_ETHERNET_IP_NATIVE_LIB}. Run cargo build --release --features ffi --locked first.")
endif()

add_library(rust_ethernet_ip_native SHARED IMPORTED GLOBAL)
set_target_properties(rust_ethernet_ip_native PROPERTIES
    IMPORTED_LOCATION "${RUST_ETHERNET_IP_NATIVE_LIB}")

if(WIN32)
    get_filename_component(NATIVE_DIR "${RUST_ETHERNET_IP_NATIVE_LIB}" DIRECTORY)
    set(NATIVE_IMPORT_LIB "${NATIVE_DIR}/rust_ethernet_ip.dll.lib")
    if(NOT EXISTS "${NATIVE_IMPORT_LIB}")
        message(FATAL_ERROR "Windows import library not found: ${NATIVE_IMPORT_LIB}")
    endif()
    set_target_properties(rust_ethernet_ip_native PROPERTIES
        IMPORTED_IMPLIB "${NATIVE_IMPORT_LIB}")
endif()

add_library(cpp_warnings INTERFACE)
if(MSVC)
    target_compile_options(cpp_warnings INTERFACE /W4 /permissive-)
else()
    target_compile_options(cpp_warnings INTERFACE -Wall -Wextra -Wpedantic)
endif()

add_executable(ffi_header_link_check parity_link_check.cpp)
target_include_directories(ffi_header_link_check PRIVATE "${REPO_ROOT}/include")
target_link_libraries(ffi_header_link_check PRIVATE rust_ethernet_ip_native cpp_warnings)

add_executable(cpp_smoke_demo demo.cpp)
target_include_directories(cpp_smoke_demo PRIVATE "${REPO_ROOT}/include" "${CMAKE_CURRENT_LIST_DIR}")
target_link_libraries(cpp_smoke_demo PRIVATE rust_ethernet_ip_native cpp_warnings)

# Full-coverage exerciser (parity with the Rust/C#/Python runners). Needs a live PLC,
# so it is a build target but not a ctest.
add_executable(cpp_full_coverage full_coverage.cpp)
target_include_directories(cpp_full_coverage PRIVATE "${REPO_ROOT}/include")
target_link_libraries(cpp_full_coverage PRIVATE rust_ethernet_ip_native cpp_warnings)

if(UNIX AND NOT APPLE)
    set_target_properties(ffi_header_link_check cpp_smoke_demo cpp_full_coverage PROPERTIES BUILD_RPATH "$ORIGIN")
elseif(APPLE)
    set_target_properties(ffi_header_link_check cpp_smoke_demo cpp_full_coverage PROPERTIES BUILD_RPATH "@loader_path")
endif()

foreach(target ffi_header_link_check cpp_smoke_demo cpp_full_coverage)
    add_custom_command(TARGET ${target} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${RUST_ETHERNET_IP_NATIVE_LIB}"
                "$<TARGET_FILE_DIR:${target}>")
endforeach()

find_package(Python3 COMPONENTS Interpreter REQUIRED)
enable_testing()

add_test(NAME ffi_header_link_check COMMAND ffi_header_link_check)
add_test(
    NAME cpp_smoke_demo
    COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_LIST_DIR}/smoke.py" "$<TARGET_FILE:cpp_smoke_demo>")
set_tests_properties(cpp_smoke_demo PROPERTIES
    ENVIRONMENT "RUST_ETHERNET_IP_REPO_ROOT=${REPO_ROOT}")
