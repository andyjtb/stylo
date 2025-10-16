cmake_minimum_required(VERSION 3.15)
project(StyloFFI VERSION 1.0 LANGUAGES CXX)

# Set C++ standard
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Option to build examples
option(BUILD_EXAMPLES "Build example programs" ON)

# Find Cargo for building Rust library
find_program(CARGO_EXECUTABLE cargo REQUIRED)
message(STATUS "Found Cargo: ${CARGO_EXECUTABLE}")

# Set Rust build directory relative to the project root
set(RUST_PROJECT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../..")
set(RUST_TARGET_DIR "${RUST_PROJECT_DIR}/target")
set(RUST_BUILD_TYPE "debug")  # Can be changed to "release"

# Custom command to build the Rust library
add_custom_command(
    OUTPUT ${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/libstyle.a
    COMMAND ${CARGO_EXECUTABLE} build $<$<CONFIG:Release>:--release>
    WORKING_DIRECTORY ${RUST_PROJECT_DIR}
    COMMENT "Building Rust Stylo library..."
    VERBATIM
)

# Custom target for Rust library
add_custom_target(stylo_rust_lib ALL
    DEPENDS ${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/libstyle.a
)

# Function to find the Stylo build directory (it has a hash in the name)
function(find_stylo_build_dir OUTPUT_VAR)
    file(GLOB STYLO_BUILD_DIRS "${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/build/stylo-*")
    list(GET STYLO_BUILD_DIRS 0 STYLO_BUILD_DIR)
    if(STYLO_BUILD_DIR)
        set(${OUTPUT_VAR} ${STYLO_BUILD_DIR} PARENT_SCOPE)
        message(STATUS "Found Stylo build directory: ${STYLO_BUILD_DIR}")
    else()
        message(FATAL_ERROR "Could not find Stylo build directory")
    endif()
endfunction()

# Function to find the cxx bridge library directory
function(find_cxx_lib_dir OUTPUT_VAR)
    file(GLOB_RECURSE CXX_LIB "${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/build/cxx-*/out/libcxxbridge1.a")
    list(GET CXX_LIB 0 CXX_LIB_PATH)
    if(CXX_LIB_PATH)
        get_filename_component(CXX_LIB_DIR ${CXX_LIB_PATH} DIRECTORY)
        set(${OUTPUT_VAR} ${CXX_LIB_DIR} PARENT_SCOPE)
        message(STATUS "Found cxxbridge library directory: ${CXX_LIB_DIR}")
    else()
        message(FATAL_ERROR "Could not find cxxbridge library")
    endif()
endfunction()

# Find the build directories after Rust library is built
find_stylo_build_dir(STYLO_BUILD_DIR)
find_cxx_lib_dir(CXX_LIB_DIR)

# Copy selector_bridge.h to cxxbridge include directory after building Rust library
add_custom_command(
    TARGET stylo_rust_lib POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory ${STYLO_BUILD_DIR}/out/cxxbridge/include/stylo/ffi
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_CURRENT_SOURCE_DIR}/selector_bridge.h
        ${STYLO_BUILD_DIR}/out/cxxbridge/include/stylo/ffi/selector_bridge.h
    COMMENT "Copying selector_bridge.h to cxxbridge include directory..."
    VERBATIM
)

# Create an interface library for Stylo
add_library(stylo_ffi INTERFACE)
add_dependencies(stylo_ffi stylo_rust_lib)

# Set include directories as SYSTEM to avoid source directory issues
target_include_directories(stylo_ffi SYSTEM INTERFACE
    $<BUILD_INTERFACE:${STYLO_BUILD_DIR}/out/cxxbridge/include>
    $<INSTALL_INTERFACE:include>
)

# Get absolute paths for libraries
set(STYLE_LIB "${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}/libstyle.a")
set(CXXBRIDGE_LIB "${CXX_LIB_DIR}/libcxxbridge1.a")

# Link libraries directly with absolute paths
if(APPLE)
    target_link_libraries(stylo_ffi INTERFACE
        -Wl,-force_load ${STYLE_LIB}
        ${CXXBRIDGE_LIB}
        pthread
        dl
        m
    )
elseif(UNIX)
    target_link_libraries(stylo_ffi INTERFACE
        -Wl,--whole-archive ${STYLE_LIB} -Wl,--no-whole-archive
        ${CXXBRIDGE_LIB}
        pthread
        dl
        m
        gcc_s
    )
else()
    # Windows
    target_link_libraries(stylo_ffi INTERFACE
        ${STYLE_LIB}
        ${CXXBRIDGE_LIB}
    )
endif()

# Build examples if requested
if(BUILD_EXAMPLES)
    # Example 1: General FFI demo
    add_executable(example example.cpp)
    target_link_libraries(example PRIVATE stylo_ffi)
    
    # Example 2: Color parser
    add_executable(color_parser color_parser.cpp)
    target_link_libraries(color_parser PRIVATE stylo_ffi)
    
    # Example 3: Selector matching (provides its own implementations)
    # Note: Uses -Wl,--allow-multiple-definition to override stub implementations
    add_executable(selector_example selector_example.cpp)
    if(UNIX AND NOT APPLE)
        target_link_options(selector_example PRIVATE -Wl,--allow-multiple-definition)
    endif()
    target_link_libraries(selector_example PRIVATE stylo_ffi)
    
    message(STATUS "Examples will be built: example, color_parser, selector_example")
endif()

# Installation rules
install(TARGETS stylo_ffi
    EXPORT StyloFFITargets
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
    INCLUDES DESTINATION include
)

# Export the targets
install(EXPORT StyloFFITargets
    FILE StyloFFITargets.cmake
    NAMESPACE Stylo::
    DESTINATION lib/cmake/StyloFFI
)

# Create and install the config file
include(CMakePackageConfigHelpers)
configure_package_config_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/StyloFFIConfig.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/StyloFFIConfig.cmake
    INSTALL_DESTINATION lib/cmake/StyloFFI
)

write_basic_package_version_file(
    ${CMAKE_CURRENT_BINARY_DIR}/StyloFFIConfigVersion.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY AnyNewerVersion
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/StyloFFIConfig.cmake
    ${CMAKE_CURRENT_BINARY_DIR}/StyloFFIConfigVersion.cmake
    DESTINATION lib/cmake/StyloFFI
)

# Print configuration summary
message(STATUS "")
message(STATUS "Stylo FFI Configuration:")
message(STATUS "  C++ Standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  Build Type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  Rust Target Dir: ${RUST_TARGET_DIR}")
message(STATUS "  Stylo Build Dir: ${STYLO_BUILD_DIR}")
message(STATUS "  CXX Lib Dir: ${CXX_LIB_DIR}")
message(STATUS "  Build Examples: ${BUILD_EXAMPLES}")
message(STATUS "")
