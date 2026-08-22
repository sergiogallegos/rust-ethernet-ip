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

if(APPLE)
    # Rust's cdylib may carry an absolute LC_ID_DYLIB pointing into Cargo's
    # deps directory. Capture it so each example can be rewritten to load the
    # copy placed beside the executable instead of a stale Cargo stub.
    execute_process(
        COMMAND otool -D "${RUST_ETHERNET_IP_NATIVE_LIB}"
        OUTPUT_VARIABLE NATIVE_OTOOL_OUTPUT
        OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY)
    string(REPLACE "\n" ";" NATIVE_OTOOL_LINES "${NATIVE_OTOOL_OUTPUT}")
    list(LENGTH NATIVE_OTOOL_LINES NATIVE_OTOOL_LINE_COUNT)
    if(NATIVE_OTOOL_LINE_COUNT GREATER 1)
        list(GET NATIVE_OTOOL_LINES 1 NATIVE_INSTALL_NAME)
        string(STRIP "${NATIVE_INSTALL_NAME}" NATIVE_INSTALL_NAME)
    else()
        message(FATAL_ERROR "Could not determine install name for ${RUST_ETHERNET_IP_NATIVE_LIB}")
    endif()
endif()

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

add_executable(cpp_route_and_diagnostics route_and_diagnostics.cpp)
target_include_directories(cpp_route_and_diagnostics PRIVATE "${REPO_ROOT}/include")
target_link_libraries(cpp_route_and_diagnostics PRIVATE rust_ethernet_ip_native cpp_warnings)

add_executable(cpp_discovery discovery.cpp)
target_include_directories(cpp_discovery PRIVATE "${REPO_ROOT}/include")
target_link_libraries(cpp_discovery PRIVATE rust_ethernet_ip_native cpp_warnings)

add_executable(cpp_udt_and_scope udt_and_scope.cpp)
target_include_directories(cpp_udt_and_scope PRIVATE "${REPO_ROOT}/include")
target_link_libraries(cpp_udt_and_scope PRIVATE rust_ethernet_ip_native cpp_warnings)

# Full-coverage exerciser (parity with the Rust/C#/Python runners). Needs a live PLC,
# so it is a build target but not a ctest.
add_executable(cpp_full_coverage full_coverage.cpp)
target_include_directories(cpp_full_coverage PRIVATE "${REPO_ROOT}/include")
target_link_libraries(cpp_full_coverage PRIVATE rust_ethernet_ip_native cpp_warnings)

# Restore-safe batch / whole-UDT / discovery companion gate. Live mode needs a
# PLC; CI only invokes its --dry-run path.
add_executable(cpp_hardware_feature_gate hardware_feature_gate.cpp)
target_include_directories(cpp_hardware_feature_gate PRIVATE "${REPO_ROOT}/include")
target_link_libraries(cpp_hardware_feature_gate PRIVATE rust_ethernet_ip_native cpp_warnings)

# Live companion for docs/validation/SCHEMA_CHANGE_GATE.md. Live mode needs a
# PLC and a maintainer at Studio 5000; CI only invokes its --dry-run path.
add_executable(cpp_schema_gate_live schema_change_gate_live.cpp)
target_include_directories(cpp_schema_gate_live PRIVATE "${REPO_ROOT}/include")
target_link_libraries(cpp_schema_gate_live PRIVATE rust_ethernet_ip_native cpp_warnings)

if(UNIX AND NOT APPLE)
    set_target_properties(ffi_header_link_check cpp_smoke_demo cpp_route_and_diagnostics cpp_discovery cpp_udt_and_scope cpp_full_coverage cpp_hardware_feature_gate cpp_schema_gate_live PROPERTIES BUILD_RPATH "$ORIGIN")
elseif(APPLE)
    set_target_properties(ffi_header_link_check cpp_smoke_demo cpp_route_and_diagnostics cpp_discovery cpp_udt_and_scope cpp_full_coverage cpp_hardware_feature_gate cpp_schema_gate_live PROPERTIES BUILD_RPATH "@loader_path")
endif()

foreach(target ffi_header_link_check cpp_smoke_demo cpp_route_and_diagnostics cpp_discovery cpp_udt_and_scope cpp_full_coverage cpp_hardware_feature_gate cpp_schema_gate_live)
    add_custom_command(TARGET ${target} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${RUST_ETHERNET_IP_NATIVE_LIB}"
                "$<TARGET_FILE_DIR:${target}>")
    if(APPLE)
        add_custom_command(TARGET ${target} POST_BUILD
            COMMAND install_name_tool -change
                    "${NATIVE_INSTALL_NAME}"
                    "@loader_path/librust_ethernet_ip.dylib"
                    "$<TARGET_FILE:${target}>")
    endif()
endforeach()

find_package(Python3 COMPONENTS Interpreter REQUIRED)
enable_testing()

add_test(NAME ffi_header_link_check COMMAND ffi_header_link_check)
add_test(
    NAME cpp_smoke_demo
    COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_LIST_DIR}/smoke.py" "$<TARGET_FILE:cpp_smoke_demo>")
set_tests_properties(cpp_smoke_demo PROPERTIES
    ENVIRONMENT "RUST_ETHERNET_IP_REPO_ROOT=${REPO_ROOT}")
add_test(NAME cpp_hardware_feature_gate_dry_run COMMAND cpp_hardware_feature_gate --dry-run)
add_test(NAME cpp_schema_gate_live_dry_run COMMAND cpp_schema_gate_live --dry-run)
