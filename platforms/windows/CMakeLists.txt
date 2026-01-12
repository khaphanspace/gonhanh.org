cmake_minimum_required(VERSION 3.22)
project(gonhanh-windows LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# MSVC static CRT (critical for Rust compatibility)
if(MSVC)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
endif()

# Corrosion for Rust integration
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5
)
FetchContent_MakeAvailable(Corrosion)

# Import Rust crate
corrosion_import_crate(
    MANIFEST_PATH ${CMAKE_SOURCE_DIR}/../../core/Cargo.toml
    CRATES gonhanh-core
    CRATE_TYPES staticlib
)

# Main executable
add_executable(gonhanh WIN32
    src/main.cpp
    src/keyboard_hook.cpp
    src/rust_bridge.cpp
    src/settings.cpp
    src/system_tray.cpp
    src/settings_window.cpp
    src/shortcuts_dialog.cpp
    src/about_dialog.cpp
    src/app_compat.cpp
    src/per_app.cpp
    src/utils.cpp
    resources/resources.rc
)

# Link Rust static lib + system libs
target_link_libraries(gonhanh PRIVATE
    gonhanh-core
    ws2_32 userenv bcrypt advapi32
    user32 shell32 gdi32 comctl32 psapi
)

# Include directories
target_include_directories(gonhanh PRIVATE
    ${CMAKE_SOURCE_DIR}/src
)

# Size optimization
if(MSVC)
    target_compile_options(gonhanh PRIVATE /O1 /GL /GS-)
    target_link_options(gonhanh PRIVATE /LTCG /OPT:REF /OPT:ICF)
endif()
