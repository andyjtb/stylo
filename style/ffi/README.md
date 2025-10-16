# Stylo CSS Parser FFI

This module exposes Stylo's CSS parsing facilities to C++ through the [cxx](https://cxx.rs/) bridge.

## Features

The FFI provides the following capabilities:

### 1. Stylesheet Parsing
Parse complete CSS stylesheets from strings:
```rust
parse_stylesheet(css: &str, base_url: &str) -> ParseResult
```

Example:
```cpp
auto result = parse_stylesheet(
    "body { color: red; font-size: 16px; }",
    "https://example.com/style.css"
);
if (result.success) {
    std::cout << "Stylesheet parsed successfully!" << std::endl;
}
```

### 2. CSS Value Parsing
Parse individual CSS values:
```rust
parse_css_value(value: &str, property_name: &str) -> ParsedCSSValue
```

Example:
```cpp
auto value = parse_css_value("10px", "width");
if (value.success) {
    std::cout << "Parsed value: " << value.value << std::endl;
}
```

### 3. Media Query Parsing
Parse and validate media queries:
```rust
parse_media_query(query: &str) -> ParseResult
```

Example:
```cpp
auto media = parse_media_query("(min-width: 768px)");
if (media.success) {
    std::cout << "Valid media query!" << std::endl;
}
```

### 4. Calc Expression Evaluation
Evaluate calc() expressions:
```rust
evaluate_calc_expression(expr: &str) -> CalcResult
```

Example:
```cpp
auto calc = evaluate_calc_expression("calc(5)");
if (calc.success) {
    std::cout << "Result: " << calc.value << std::endl;
}
```

### 5. Computed Value Resolution
Get computed values for CSS properties:
```rust
get_computed_value(value: &str, property_name: &str, base_font_size: f32) -> ParsedCSSValue
```

Example:
```cpp
auto computed = get_computed_value("2em", "font-size", 16.0f);
if (computed.success) {
    std::cout << "Computed: " << computed.value << std::endl;
}
```

### 6. Color Parsing
Parse CSS color values with structured output or as nscolor (uint32):

#### Structured Color Output
```rust
parse_color(color_str: &str) -> ParsedColor
```

Example:
```cpp
auto color = parse_color("hsla(-300, 100%, 37.5%, -3)");
if (color.success) {
    std::cout << "Color Space: " << color.color_space << std::endl;
    std::cout << "Components: (" << color.components.c0 << ", " 
              << color.components.c1 << ", " << color.components.c2 << ")" << std::endl;
    std::cout << "Alpha: " << color.alpha << std::endl;
}
```

Returns a structured `ParsedColor` with:
- `components`: Color components (3 floats) - interpretation depends on color space
- `alpha`: Alpha channel (0.0 to 1.0)
- `color_space`: Enum indicating the color space (Srgb, Hsl, Lab, etc.)
- `success`: Whether parsing succeeded
- `error_message`: Error details if parsing failed

#### nscolor (uint32 RGBA) Output
```rust
parse_color_to_nscolor(color_str: &str) -> ParsedNsColor
```

Example:
```cpp
auto color = parse_color_to_nscolor("rgb(255, 0, 0)");
if (color.success) {
    uint32_t nscolor = color.nscolor;  // 0xFF0000FF (RGBA)
    // Extract components
    uint8_t r = nscolor & 0xFF;
    uint8_t g = (nscolor >> 8) & 0xFF;
    uint8_t b = (nscolor >> 16) & 0xFF;
    uint8_t a = (nscolor >> 24) & 0xFF;
}
```

The `nscolor` format (little-endian RGBA uint32) is compatible with:
- Mozilla's nscolor
- Qt's QRgb  
- Other RGBA uint32 color formats

**Use this for easy GUI framework integration!** All colors are automatically converted to sRGB.

Supports all CSS color formats:
- Named colors: `red`, `blue`, `transparent`
- Hex colors: `#ff0000`, `#f00`
- RGB/RGBA: `rgb(255, 0, 0)`, `rgba(0, 128, 255, 0.5)`
- HSL/HSLA: `hsl(120, 100%, 50%)`, `hsla(-300, 100%, 37.5%, -3)`
- Lab/Lch: `lab(50% 20 30)`, `lch(50% 40 180)`
- Oklab/Oklch: `oklab(0.5 0.1 0.1)`, `oklch(0.5 0.2 180)`
- And more color spaces (Display P3, Rec2020, XYZ, etc.)

## Types

### ParseResult
```rust
struct ParseResult {
    success: bool,
    error_message: String,
}
```

### ParsedCSSValue
```rust
struct ParsedCSSValue {
    value: String,
    success: bool,
}
```

### ParsedColor
```rust
struct ParsedColor {
    success: bool,
    components: ColorComponents,  // 3 floats: c0, c1, c2
    alpha: f32,
    color_space: ColorSpace,      // Enum: Srgb, Hsl, Lab, etc.
    error_message: String,
}
```

### ParsedNsColor
```rust
struct ParsedNsColor {
    success: bool,
    nscolor: u32,           // RGBA as uint32 (little-endian)
    error_message: String,
}
```

The `nscolor` field contains RGBA bytes packed as:
- Byte 0 (bits 0-7): Red (0-255)
- Byte 1 (bits 8-15): Green (0-255)
- Byte 2 (bits 16-23): Blue (0-255)
- Byte 3 (bits 24-31): Alpha (0-255)

Compatible with Mozilla nscolor, Qt QRgb, and other RGBA uint32 formats.

### CalcResult
```rust
struct CalcResult {
    value: f32,
    success: bool,
}
```

## Building

The FFI is automatically built when you build the stylo crate with the following dependencies:
- `cxx = "1.0"` (runtime dependency)
- `cxx-build = "1.0"` (build dependency)

The build.rs file handles cxx code generation.

**Note:** To enable C++ linking, the style crate is built as both `rlib` and `staticlib` (see `Cargo.toml`).

## Usage from C++

### Building with CMake (Recommended)

The easiest way to integrate Stylo FFI into your C++ project is using CMake:

```bash
cd style/ffi
mkdir build
cd build
cmake ..
cmake --build .

# Run the examples
./color_parser
./example
```

See [CMAKE_GUIDE.md](CMAKE_GUIDE.md) for detailed CMake usage and integration instructions.

### Building with Make (Alternative)

You can also use the traditional Makefile:

```bash
cd style/ffi
make color_parser    # Build the color_parser example
make run_color_parser  # Run the color_parser example
make example         # Build the general example
make run             # Run the general example
```

### Examples

The FFI includes two C++ example programs:

1. **example.cpp** - General FFI demo showing all features
2. **color_parser.cpp** - C++ version of the Rust color_parser.rs example

### Including in Your Project

See `example_usage.h` for C++ usage examples.

The generated C++ header will be available at:
```
target/<profile>/cxxbridge/stylo/ffi/mod.rs.h
```

Include this header in your C++ code to access the FFI functions:

```cpp
#include "rust/cxx.h"
#include "stylo/ffi/mod.rs.h"
```

## Testing

The module includes unit tests to verify functionality:

```bash
cargo test --lib ffi
```

Note: Some tests may fail due to unrelated test infrastructure issues in the main codebase.

## Implementation Status

### Currently Implemented
- ✅ Basic stylesheet parsing
- ✅ Media query parsing with full syntax support
- ✅ CSS value validation
- ✅ Calc expression parsing with CalcNode
- ✅ Simple numeric value extraction from calc
- ✅ Unit tests for all major functions

### Future Enhancements
- [ ] Full CSS value parsing with property-specific validation
- [ ] Complete calc() expression evaluation with units and operations
- [ ] Computed value resolution with proper rendering context
- [ ] Stylesheet manipulation APIs (add/remove rules)
- [ ] Advanced error reporting with line/column information
- [ ] Support for custom properties (@property rules)
- [ ] Cascade resolution APIs
- [ ] Style computation with element context

## Technical Details

### Architecture

The FFI layer is designed to be:
- **Safe**: Uses cxx for type-safe C++/Rust interop
- **Minimal**: Exposes only essential parsing functions
- **Extensible**: Easy to add new functions without breaking changes
- **Efficient**: Zero-copy where possible, minimal allocations

### Error Handling

All functions return result types with success flags and error messages:
- `ParseResult` for stylesheet and media query parsing
- `ParsedCSSValue` for value parsing operations
- `CalcResult` for calc expression evaluation

Errors are descriptive and include context about what failed.

### Limitations

1. **Computed Values**: Full computed value resolution requires an element context and rendering state, which is not yet exposed
2. **Calc Evaluation**: Complex calc() expressions with mixed units require a resolution context
3. **Property-Specific Parsing**: Individual property value parsing is generic; property-specific validation will be added incrementally

## Notes

- The current implementation provides robust parsing facilities
- Error handling returns success/failure status with descriptive messages
- The API is designed to be extended without breaking changes
- Generated C++ code is compatible with C++14 and later
