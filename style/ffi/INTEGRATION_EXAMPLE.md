# Example: Integrating Stylo FFI into Your C++ Project

This directory demonstrates how to use the Stylo FFI in your own C++ project using CMake.

## Project Structure

```
my_project/
├── CMakeLists.txt          # Your project's CMake configuration
├── main.cpp                # Your application code
└── path/to/stylo/          # The Stylo repository
```

## Example CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyProject)

# Add Stylo FFI as a subdirectory
add_subdirectory(path/to/stylo/style/ffi stylo_ffi)

# Create your application
add_executable(my_app main.cpp)

# Link against Stylo FFI
target_link_libraries(my_app PRIVATE stylo_ffi)
```

## Example main.cpp

```cpp
#include <iostream>
#include <string>
#include "stylo/ffi/mod.rs.h"

int main() {
    // Parse a stylesheet
    auto stylesheet = parse_stylesheet(
        "body { background: #fff; color: #000; }",
        "https://example.com/style.css"
    );
    
    if (stylesheet.success) {
        std::cout << "Stylesheet parsed successfully!" << std::endl;
    } else {
        std::cout << "Error: " << std::string(stylesheet.error_message) << std::endl;
    }
    
    // Parse a color
    auto color = parse_color("rgb(255, 128, 0)");
    
    if (color.success) {
        std::cout << "Color: " << std::string(color.value) << std::endl;
    } else {
        std::cout << "Failed to parse color" << std::endl;
    }
    
    // Parse a media query
    auto media = parse_media_query("(min-width: 768px) and (max-width: 1024px)");
    
    if (media.success) {
        std::cout << "Media query is valid!" << std::endl;
    } else {
        std::cout << "Invalid media query: " << std::string(media.error_message) << std::endl;
    }
    
    return 0;
}
```

## Building

```bash
mkdir build
cd build
cmake ..
cmake --build .
./my_app
```

## Using as an Installed Package

If you've installed Stylo FFI system-wide:

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyProject)

# Find the installed Stylo FFI package
find_package(StyloFFI REQUIRED)

# Create your application
add_executable(my_app main.cpp)

# Link against Stylo FFI
target_link_libraries(my_app PRIVATE Stylo::stylo_ffi)
```

## Installation

To install Stylo FFI system-wide:

```bash
cd path/to/stylo/style/ffi
mkdir build && cd build
cmake ..
cmake --build .
sudo cmake --install . --prefix /usr/local
```

Then in your project, CMake will automatically find it with `find_package(StyloFFI REQUIRED)`.
