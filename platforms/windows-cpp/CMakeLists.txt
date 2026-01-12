cmake_minimum_required(VERSION 3.20)
project(gonhanh VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Static CRT linking for single .exe
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

# Compiler flags for optimization and security
if(MSVC)
    add_compile_options(
        /W4                  # Warning level 4
        /WX                  # Warnings as errors
        /permissive-         # Standards conformance
        /Zc:__cplusplus      # Enable __cplusplus macro
        /GS                  # Buffer security check
        /sdl                 # Additional security checks
        $<$<CONFIG:Release>:/O2>  # Optimize for speed
        $<$<CONFIG:Release>:/GL>  # Whole program optimization
    )
    add_link_options(
        $<$<CONFIG:Release>:/LTCG>      # Link-time code generation
        $<$<CONFIG:Release>:/OPT:REF>   # Remove unreferenced functions
        $<$<CONFIG:Release>:/OPT:ICF>   # Identical COMDAT folding
    )
endif()

# Source files
set(SOURCES
    src/main.cpp
    src/RustBridge.cpp
    src/KeyboardHook.cpp
    src/TextSender.cpp
)

# Header files (for IDE)
set(HEADERS
    src/RustBridge.h
    src/KeyboardHook.h
    src/TextSender.h
    src/LockFreeQueue.h
    src/KeycodeMap.h
    src/Types.h
)

# Create executable (WIN32 = no console window)
add_executable(gonhanh WIN32 ${SOURCES} ${HEADERS})

# Include directories
target_include_directories(gonhanh PRIVATE src)

# Link Rust core library
target_link_libraries(gonhanh PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/lib/libgonhanh_core.lib
    kernel32
    user32
    shell32
)

# Copy DLL to output directory after build
add_custom_command(TARGET gonhanh POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${CMAKE_CURRENT_SOURCE_DIR}/lib/gonhanh_core.dll"
        "$<TARGET_FILE_DIR:gonhanh>"
    COMMENT "Copying gonhanh_core.dll to output directory"
)

# Set output directory
set_target_properties(gonhanh PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/bin"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/bin"
)

# Installation (optional)
install(TARGETS gonhanh DESTINATION bin)
install(FILES lib/gonhanh_core.dll DESTINATION bin)
