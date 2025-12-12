cmake_minimum_required(VERSION 3.16)
project(fcitx5-gonhanh VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find Fcitx5 (Ubuntu package names: Fcitx5Core, Fcitx5Module)
find_package(PkgConfig REQUIRED)
pkg_check_modules(Fcitx5Core REQUIRED IMPORTED_TARGET Fcitx5Core)
pkg_check_modules(Fcitx5Module REQUIRED IMPORTED_TARGET Fcitx5Module)

# Rust core library path
set(RUST_LIB_DIR "${CMAKE_SOURCE_DIR}/../../core/target/release")
set(RUST_LIB_NAME "gonhanh_core")

# Check if Rust library exists
if(NOT EXISTS "${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.so")
    message(WARNING "Rust library not found at ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.so")
    message(WARNING "Build it first: cd ../../core && cargo build --release")
endif()

# Sources (KeycodeMap.h is header-only)
set(SOURCES
    src/Engine.cpp
    src/RustBridge.cpp
)

# Create addon shared library
add_library(gonhanh MODULE ${SOURCES})

# Include directories
target_include_directories(gonhanh PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${RUST_LIB_DIR}
)

# Link libraries
target_link_libraries(gonhanh
    PkgConfig::Fcitx5Core
    PkgConfig::Fcitx5Module
    ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.so
)

# Set output properties
set_target_properties(gonhanh PROPERTIES
    PREFIX ""
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}"
    # Set RPATH so addon can find libgonhanh_core.so in ~/.local/lib
    BUILD_RPATH "$ORIGIN/../../lib;$ORIGIN/../.."
    INSTALL_RPATH "$ORIGIN/../../lib;$ORIGIN/../.."
)

# Get Fcitx5 addon install paths (with fallback for CI)
pkg_get_variable(FCITX5_ADDON_DIR Fcitx5Core addondir)
pkg_get_variable(FCITX5_LIB_DIR Fcitx5Core libdir)

# Fallback to standard paths if pkg-config doesn't provide them
if(NOT FCITX5_ADDON_DIR)
    set(FCITX5_ADDON_DIR "${CMAKE_INSTALL_PREFIX}/share/fcitx5/addon")
endif()
if(NOT FCITX5_LIB_DIR)
    set(FCITX5_LIB_DIR "${CMAKE_INSTALL_PREFIX}/lib")
endif()

# Install targets
install(TARGETS gonhanh
    LIBRARY DESTINATION "${FCITX5_LIB_DIR}/fcitx5"
)

install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/data/gonhanh-addon.conf"
    DESTINATION "${FCITX5_ADDON_DIR}"
    RENAME "gonhanh.conf"
)

install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/data/gonhanh.conf"
    DESTINATION "${FCITX5_ADDON_DIR}/inputmethod"
)

install(FILES "${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.so"
    DESTINATION "${FCITX5_LIB_DIR}"
)

# User-local install target (no sudo needed)
add_custom_target(install-user
    COMMAND ${CMAKE_COMMAND} -E make_directory "$ENV{HOME}/.local/lib/fcitx5"
    COMMAND ${CMAKE_COMMAND} -E make_directory "$ENV{HOME}/.local/share/fcitx5/addon"
    COMMAND ${CMAKE_COMMAND} -E make_directory "$ENV{HOME}/.local/share/fcitx5/inputmethod"
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_BINARY_DIR}/gonhanh.so" "$ENV{HOME}/.local/lib/fcitx5/"
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_CURRENT_SOURCE_DIR}/data/gonhanh-addon.conf" "$ENV{HOME}/.local/share/fcitx5/addon/gonhanh.conf"
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_CURRENT_SOURCE_DIR}/data/gonhanh.conf" "$ENV{HOME}/.local/share/fcitx5/inputmethod/"
    COMMAND ${CMAKE_COMMAND} -E copy "${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.so" "$ENV{HOME}/.local/lib/"
    COMMENT "Installing to user-local Fcitx5 paths"
    DEPENDS gonhanh
)

message(STATUS "Fcitx5 addon dir: ${FCITX5_ADDON_DIR}")
message(STATUS "Fcitx5 lib dir: ${FCITX5_LIB_DIR}")
message(STATUS "Rust core lib: ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.so")

# =============================================================================
# Testing Configuration (optional - only if GTest is available)
# =============================================================================
option(BUILD_TESTS "Build unit tests" OFF)

if(BUILD_TESTS)
    find_package(GTest QUIET)

    if(GTest_FOUND)
        enable_testing()
        include(GoogleTest)

        # KeycodeMap tests (header-only, no Fcitx5 dependencies)
        add_executable(keycodemap_test tests/KeycodeMapTest.cpp)
        target_include_directories(keycodemap_test PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/src
        )
        target_link_libraries(keycodemap_test GTest::gtest GTest::gtest_main)
        gtest_discover_tests(keycodemap_test)

        # RustBridge UTF-8 conversion tests
        add_executable(rustbridge_test tests/RustBridgeTest.cpp src/RustBridge.cpp)
        target_include_directories(rustbridge_test PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${RUST_LIB_DIR}
        )
        target_link_libraries(rustbridge_test
            GTest::gtest
            GTest::gtest_main
            ${RUST_LIB_DIR}/lib${RUST_LIB_NAME}.so
        )
        set_target_properties(rustbridge_test PROPERTIES
            BUILD_RPATH "$ORIGIN/../../lib;$ORIGIN/../..;${RUST_LIB_DIR}"
        )
        gtest_discover_tests(rustbridge_test)

        message(STATUS "Tests enabled - will build keycodemap_test and rustbridge_test")
    else()
        message(WARNING "GTest not found - tests will not be built")
        message(WARNING "Install with: sudo apt install libgtest-dev")
    endif()
endif()
