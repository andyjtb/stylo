# Building with CMake

This directory contains a CMake build system for the Stylo FFI C++ bindings.

## Prerequisites

- CMake 3.15 or later
- Rust toolchain (cargo)
- C++ compiler with C++14 support
- Standard build tools (make, ninja, or Visual Studio)

## Quick Start

### Building the Examples

```bash
# From the style/ffi directory
mkdir build
cd build
cmake ..
cmake --build .

# Run the examples
./color_parser
./example
```

### Building in Release Mode

```bash
cmake -DCMAKE_BUILD_TYPE=Release ..
cmake --build .
```

### Building Without Examples

```bash
cmake -DBUILD_EXAMPLES=OFF ..
cmake --build .
```

## Using Stylo FFI in Your C++ Project

### Method 1: Using CMake's add_subdirectory

If you've placed the Stylo repository as a subdirectory:

```cmake
# In your CMakeLists.txt
add_subdirectory(path/to/stylo/style/ffi)

add_executable(my_app main.cpp)
target_link_libraries(my_app PRIVATE stylo_ffi)
```

### Method 2: Using find_package (after installation)

```bash
# Install Stylo FFI
cd build
cmake --install . --prefix /usr/local
```

Then in your project:

```cmake
# In your CMakeLists.txt
find_package(StyloFFI REQUIRED)

add_executable(my_app main.cpp)
target_link_libraries(my_app PRIVATE Stylo::stylo_ffi)
```

### Method 3: Manual Integration

```cmake
# Set paths to Stylo
set(STYLO_ROOT "/path/to/stylo")
set(STYLO_TARGET_DIR "${STYLO_ROOT}/target/debug")

# Find the build directories (you'll need to update the hash)
set(STYLO_BUILD_DIR "${STYLO_TARGET_DIR}/build/stylo-<hash>")
set(CXX_BUILD_DIR "${STYLO_TARGET_DIR}/build/cxx-<hash>")

# Add include directories
include_directories(${STYLO_BUILD_DIR}/out/cxxbridge/include)

# Link libraries
link_directories(
    ${STYLO_BUILD_DIR}/out
    ${CXX_BUILD_DIR}/out
    ${STYLO_TARGET_DIR}
)

# Create your executable
add_executable(my_app main.cpp)

# Link against Stylo
if(UNIX AND NOT APPLE)
    target_link_libraries(my_app PRIVATE
        -Wl,--whole-archive style -Wl,--no-whole-archive
        cxxbridge1
        pthread dl m gcc_s
    )
elseif(APPLE)
    target_link_libraries(my_app PRIVATE
        -Wl,-force_load style
        cxxbridge1
        pthread dl m
    )
endif()
```

## Example C++ Code

```cpp
#include <iostream>
#include <string>
#include "stylo/ffi/mod.rs.h"

int main() {
    // Parse a color
    auto color = parse_color("hsla(-300, 100%, 37.5%, -3)");
    
    if (color.success) {
        std::cout << "Parsed color: " << std::string(color.value) << std::endl;
    } else {
        std::cout << "Failed to parse color" << std::endl;
    }
    
    return 0;
}
```

## Troubleshooting

### "Could not find Stylo build directory"

This means the Rust library hasn't been built yet. CMake will automatically build it, but if you see this error:

```bash
# Manually build the Rust library first
cd ../../..  # Go to stylo root
cargo build
cd style/ffi
```

### Header file not found

Make sure the Rust library has been built at least once. CMake will handle this automatically, but you can verify:

```bash
ls ../../target/debug/build/stylo-*/out/cxxbridge/include/stylo/ffi/mod.rs.h
```

### Linking errors

If you see undefined symbols, ensure you're using the whole-archive linker flags (CMake handles this automatically).

## Advanced Configuration

### Cross-Compilation

To cross-compile for a different target:

```bash
# Set Rust target
export CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu

# Configure CMake
cmake -DCMAKE_TOOLCHAIN_FILE=your_toolchain.cmake ..
```

### Custom Rust Build Type

The CMake build uses the Rust debug build by default. To use release:

```cmake
# Modify CMakeLists.txt
set(RUST_BUILD_TYPE "release")
```

Or pass it as a CMake variable:

```bash
cmake -DRUST_BUILD_TYPE=release ..
```

## Platform Support

- **Linux**: Fully supported
- **macOS**: Supported with Clang
- **Windows**: Experimental (MSVC toolchain)

## Integration with Other Build Systems

### Makefile-based Projects

You can still use the traditional Makefile alongside CMake. Both are provided for convenience.

### Meson, Bazel, etc.

While not officially supported, you can adapt the linking steps from the CMake configuration to other build systems.
