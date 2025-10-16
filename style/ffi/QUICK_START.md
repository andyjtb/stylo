# Quick Start Guide - Stylo C++ FFI

Get up and running with Stylo's C++ bindings in minutes!

## Build and Run (CMake)

```bash
cd style/ffi
mkdir build && cd build
cmake ..
cmake --build .
./color_parser  # Run the color parser example
./example       # Run the general example
```

That's it! CMake handles everything automatically:
- ✅ Builds the Rust library
- ✅ Finds all headers and libraries  
- ✅ Links everything correctly

## Use in Your Project

### Option 1: Add as Subdirectory

```cmake
# CMakeLists.txt
cmake_minimum_required(VERSION 3.15)
project(MyApp)

add_subdirectory(path/to/stylo/style/ffi)

add_executable(my_app main.cpp)
target_link_libraries(my_app PRIVATE stylo_ffi)
```

### Option 2: System Install

```bash
# Install system-wide
cd style/ffi/build
sudo cmake --install . --prefix /usr/local

# Then in your project:
find_package(StyloFFI REQUIRED)
target_link_libraries(my_app PRIVATE Stylo::stylo_ffi)
```

## Basic Usage

```cpp
#include "stylo/ffi/mod.rs.h"
#include <iostream>

int main() {
    // Parse a color
    auto color = parse_color("rgb(255, 128, 0)");
    if (color.success) {
        std::cout << "Color: " << std::string(color.value) << std::endl;
    }
    
    // Parse a stylesheet
    auto css = parse_stylesheet(
        "body { color: red; }",
        "https://example.com"
    );
    
    // Parse media query
    auto media = parse_media_query("(min-width: 768px)");
    
    return 0;
}
```

## Documentation

- **[CMAKE_GUIDE.md](CMAKE_GUIDE.md)** - Complete CMake guide
- **[INTEGRATION_EXAMPLE.md](INTEGRATION_EXAMPLE.md)** - Integration examples
- **[README.md](README.md)** - Full API documentation

## Troubleshooting

**Q: Headers not found?**  
A: Make sure Rust library is built first. CMake does this automatically.

**Q: Linker errors?**  
A: CMake handles all linking. If using manually, ensure `--whole-archive` flags.

**Q: Want to use Make instead?**  
A: `make color_parser && make run_color_parser` still works!

## Requirements

- CMake 3.15+
- Rust toolchain (cargo)
- C++14 compiler
- Linux, macOS, or Windows
