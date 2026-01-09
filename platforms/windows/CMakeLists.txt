cmake_minimum_required(VERSION 3.20)
project(GoNhanh VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Static runtime linking (no MSVC runtime dependency)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

# Source files
set(SOURCES
    src/main.cpp
    src/app.cpp
    src/logger.cpp
    src/rust_bridge.cpp
    src/keyboard_hook.cpp
    src/text_sender.cpp
    src/settings.cpp
    src/tray_icon.cpp
    src/hotkey.cpp
    src/sound_manager.cpp
    src/update_checker.cpp
    src/update_manager.cpp
    src/ui/d2d_renderer.cpp
    src/ui/validator.cpp
    src/ui/composition_window.cpp
    src/ui/settings_window.cpp
    src/ui/about_window.cpp
    src/ui/shortcuts_window.cpp
    src/ui/onboarding_window.cpp
    src/ui/update_window.cpp
    src/ui/controls/toggle.cpp
    src/ui/controls/button.cpp
    src/ui/controls/card.cpp
    src/ui/controls/sidebar.cpp
    src/ui/controls/dropdown.cpp
    src/ui/controls/text_input.cpp
    src/ui/controls/hotkey_picker.cpp
    src/ui/per_app_window.cpp
)

# Header files
set(HEADERS
    src/app.h
    src/logger.h
    src/rust_bridge.h
    src/keyboard_hook.h
    src/text_sender.h
    src/settings.h
    src/tray_icon.h
    src/hotkey.h
    src/sound_manager.h
    src/update_checker.h
    src/update_manager.h
    src/ui/d2d_renderer.h
    src/ui/validator.h
    src/ui/composition_window.h
    src/ui/settings_window.h
    src/ui/about_window.h
    src/ui/shortcuts_window.h
    src/ui/onboarding_window.h
    src/ui/update_window.h
    src/ui/controls/toggle.h
    src/ui/controls/button.h
    src/ui/controls/card.h
    src/ui/controls/sidebar.h
    src/ui/controls/dropdown.h
    src/ui/controls/text_input.h
    src/ui/controls/hotkey_picker.h
    src/ui/per_app_window.h
)

# Windows executable (no console)
add_executable(GoNhanh WIN32 ${SOURCES} ${HEADERS} resources/app.rc)

# Include directories
target_include_directories(GoNhanh PRIVATE src)

# Link Windows libraries
target_link_libraries(GoNhanh PRIVATE
    # Direct2D / DirectWrite
    d2d1
    dwrite
    windowscodecs
    # Win32 APIs
    user32
    shell32
    advapi32
    gdi32
    ole32
    comctl32
    shlwapi     # Path functions (PathCombine, etc.)
    uxtheme     # Visual styles and theming
    dwmapi      # Desktop Window Manager (blur, etc.)
    # Networking (for update system)
    winhttp
    # Audio (for sound effects)
    winmm
)

# Unicode support
target_compile_definitions(GoNhanh PRIVATE
    UNICODE
    _UNICODE
    WIN32_LEAN_AND_MEAN
    NOMINMAX
)

# Compiler settings (MSVC best practices)
if(MSVC)
    target_compile_options(GoNhanh PRIVATE
        /W4                # Warning level 4
        /WX-               # Don't treat warnings as errors (for now)
        /utf-8             # UTF-8 source and execution encoding
        /permissive-       # C++ standards conformance mode
        /Zc:__cplusplus    # Report correct __cplusplus value
        /Zc:preprocessor   # Use standards-conforming preprocessor
        /EHsc              # Standard C++ exception handling
    )
endif()

# Note: DPI awareness is set via app.manifest in app.rc
# Disable automatic manifest generation to avoid duplicate with app.rc
if(MSVC)
    set_target_properties(GoNhanh PROPERTIES LINK_FLAGS "/MANIFEST:NO")
endif()

# Output directory
set_target_properties(GoNhanh PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)
