cmake_minimum_required(VERSION 3.22)
project(gonhanh-windows LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Debug console option (for troubleshooting)
option(ENABLE_DEBUG_CONSOLE "Enable debug console window" OFF)

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

# Source files (shared between debug and release)
set(GONHANH_SOURCES
    src/main.cpp
    src/keyboard_hook.cpp
    src/rust_bridge.cpp
    src/settings.cpp
    src/system_tray.cpp
    src/settings_window.cpp
    src/ime_state.cpp
    src/cli_path.cpp
    src/autostart.cpp
    src/fluent_ui.cpp
    src/shortcuts_dialog.cpp
    src/app_compat.cpp
    src/per_app.cpp
    src/utils.cpp
    src/debug_console.cpp
    src/update_checker.cpp
    resources/resources.rc
)

# Main executable
if(ENABLE_DEBUG_CONSOLE)
    # Debug build with console window
    add_executable(gonhanh ${GONHANH_SOURCES})
    target_compile_definitions(gonhanh PRIVATE GONHANH_DEBUG_CONSOLE)
else()
    # Release build without console (WIN32)
    add_executable(gonhanh WIN32 ${GONHANH_SOURCES})
endif()

# Link Rust static lib + system libs
# Note: Corrosion converts hyphens to underscores in target names
target_link_libraries(gonhanh PRIVATE
    gonhanh_core
    ws2_32 userenv bcrypt advapi32
    user32 shell32 gdi32 comctl32 psapi
    dwmapi uxtheme winhttp
    ole32 oleaut32 taskschd
)

# App version: from -DGONHANH_VERSION=x.y.z, else the latest release tag (vX.Y.Z), else 0.0.0
set(GONHANH_VERSION "" CACHE STRING "App version shown in Settings and used by the update check")
if(NOT GONHANH_VERSION)
    execute_process(
        COMMAND git describe --tags --abbrev=0 --match "v*" --exclude "v*-pre*"
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
        OUTPUT_VARIABLE GONHANH_GIT_TAG
        OUTPUT_STRIP_TRAILING_WHITESPACE
        ERROR_QUIET)
    string(REGEX REPLACE "^v" "" GONHANH_VERSION "${GONHANH_GIT_TAG}")
endif()
if(NOT GONHANH_VERSION)
    set(GONHANH_VERSION "0.0.0")
endif()
message(STATUS "GoNhanh version: ${GONHANH_VERSION}")
# Numeric parts for the exe's version resource (Explorer → Properties → Details):
# "1.2.3" → 1,2,3,0 and a prerelease "1.2.3-pre.45" → 1,2,3,45
set(GONHANH_VERSION_NUMS 0 0 0 0)
if(GONHANH_VERSION MATCHES "^([0-9]+)\\.([0-9]+)\\.([0-9]+)(-[A-Za-z]*\\.?([0-9]+))?")
    set(GONHANH_VERSION_NUMS ${CMAKE_MATCH_1} ${CMAKE_MATCH_2} ${CMAKE_MATCH_3} 0)
    if(CMAKE_MATCH_5)
        list(REMOVE_AT GONHANH_VERSION_NUMS 3)
        list(APPEND GONHANH_VERSION_NUMS ${CMAKE_MATCH_5})
    endif()
endif()
list(GET GONHANH_VERSION_NUMS 0 GONHANH_VERSION_MAJOR)
list(GET GONHANH_VERSION_NUMS 1 GONHANH_VERSION_MINOR)
list(GET GONHANH_VERSION_NUMS 2 GONHANH_VERSION_PATCH)
list(GET GONHANH_VERSION_NUMS 3 GONHANH_VERSION_BUILD)
# Also applied to resources.rc (the resource compiler gets the target's definitions)
target_compile_definitions(gonhanh PRIVATE
    APP_VERSION_STRING="${GONHANH_VERSION}"
    APP_VERSION_MAJOR=${GONHANH_VERSION_MAJOR}
    APP_VERSION_MINOR=${GONHANH_VERSION_MINOR}
    APP_VERSION_PATCH=${GONHANH_VERSION_PATCH}
    APP_VERSION_BUILD=${GONHANH_VERSION_BUILD})

# Include directories
target_include_directories(gonhanh PRIVATE
    ${CMAKE_SOURCE_DIR}/src
)

# Unicode support (required for wide-char strings)
target_compile_definitions(gonhanh PRIVATE UNICODE _UNICODE)

# Disable auto-generated manifest (we provide our own in resources.rc)
if(MSVC)
    target_link_options(gonhanh PRIVATE /MANIFEST:NO)
endif()

# Sources are UTF-8 without BOM: without /utf-8, MSVC decodes them with the
# system ANSI code page and every Vietnamese string literal becomes mojibake
if(MSVC)
    target_compile_options(gonhanh PRIVATE /utf-8)
endif()

# Size optimization
if(MSVC)
    target_compile_options(gonhanh PRIVATE /O1 /GL)
    target_link_options(gonhanh PRIVATE /LTCG /OPT:REF /OPT:ICF)
endif()

# Command line tool `gonhanh` / `gn` (issue #433): a tiny console exe that talks to the running
# app. It goes to bin/ because Windows file names are case-insensitive: next to GoNhanh.exe,
# gonhanh.exe would be the app itself.
add_executable(gonhanh-cli cli/gonhanh_cli.cpp)
set_target_properties(gonhanh-cli PROPERTIES
    OUTPUT_NAME gonhanh
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIG>/bin"
)
target_include_directories(gonhanh-cli PRIVATE ${CMAKE_SOURCE_DIR}/src)
target_compile_definitions(gonhanh-cli PRIVATE UNICODE _UNICODE APP_VERSION_STRING="${GONHANH_VERSION}")
target_link_libraries(gonhanh-cli PRIVATE user32)
if(MSVC)
    target_compile_options(gonhanh-cli PRIVATE /utf-8 /O1 /GL)
    target_link_options(gonhanh-cli PRIVATE /LTCG /OPT:REF /OPT:ICF)
endif()
add_custom_command(TARGET gonhanh-cli POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different $<TARGET_FILE:gonhanh-cli> $<TARGET_FILE_DIR:gonhanh-cli>/gn.exe
)

# Memory: the IME lives in the tray, the UI is opened rarely. Load the UI/network DLLs only
# when the Settings/Shortcuts windows or the update check actually need them.
if(MSVC)
    target_link_libraries(gonhanh PRIVATE delayimp)
    target_link_options(gonhanh PRIVATE
        /DELAYLOAD:dwrite.dll
        /DELAYLOAD:winhttp.dll
        /DELAYLOAD:comdlg32.dll
        /DELAYLOAD:dwmapi.dll
    )
endif()
