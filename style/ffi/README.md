# Stylo CSS Parser FFI

This module exposes Stylo's CSS parsing and selector matching facilities to C++ through the [cxx](https://cxx.rs/) bridge.

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

### 6. Selector Parsing
Parse CSS selectors:
```rust
parse_selector(selector: &str) -> ParseResult
```

Example:
```cpp
auto result = parse_selector("div.my-class:hover");
if (result.success) {
    std::cout << "Valid selector!" << std::endl;
}
```

### 7. Selector Matching
Match a selector against an element:
```rust
match_selector(selector: &str, element: &FFIElement) -> SelectorMatchResult
```

Example:
```cpp
FFIElement element = /* your element */;
auto result = match_selector("div.my-class", element);
if (result.matches) {
    std::cout << "Selector matches!" << std::endl;
}
```

## Selector Matching with C++ Elements

The FFI provides a bridge for selector matching using C++ elements. To use this feature, you need to implement the required callbacks in your C++ code.

### Element State Management

Element and document states are exposed as u64 bitflags:

```cpp
// ElementState flags (from dom crate)
const uint64_t ACTIVE = 1 << 0;
const uint64_t FOCUS = 1 << 1;
const uint64_t HOVER = 1 << 2;
const uint64_t ENABLED = 1 << 3;
const uint64_t DISABLED = 1 << 4;
const uint64_t CHECKED = 1 << 5;
// ... and more

// DocumentState flags
const uint64_t WINDOW_INACTIVE = 1 << 0;
const uint64_t RTL_LOCALE = 1 << 1;
const uint64_t LTR_LOCALE = 1 << 2;
```

### Required C++ Callbacks

You must implement these callbacks in your C++ code for selector matching to work:

```cpp
// Element navigation
uint64_t get_element_state(const FFIElement& element);
uint64_t get_document_state(const FFIElement& element);
FFIElement get_parent_element(const FFIElement& element);
FFIElement get_prev_sibling_element(const FFIElement& element);
FFIElement get_next_sibling_element(const FFIElement& element);
FFIElement get_first_element_child(const FFIElement& element);

// Element properties
bool is_element_null(const FFIElement& element);
bool element_has_local_name(const FFIElement& element, rust::Str name);
bool element_has_namespace(const FFIElement& element, rust::Str ns);
bool element_has_id(const FFIElement& element, rust::Str id);
bool element_has_class(const FFIElement& element, rust::Str clazz);
bool element_is_link(const FFIElement& element);
bool element_is_root(const FFIElement& element);
bool element_is_empty(const FFIElement& element);
```

### Example Implementation

```cpp
#include "stylo/ffi/mod.rs.h"

// Your C++ element class
class MyElement {
public:
    uintptr_t id;
    std::string tag_name;
    std::vector<std::string> classes;
    uint64_t state;
    MyElement* parent;
    MyElement* prev_sibling;
    MyElement* next_sibling;
    MyElement* first_child;
};

// Implement the callbacks
uint64_t get_element_state(const FFIElement& element) {
    MyElement* elem = reinterpret_cast<MyElement*>(element.ptr);
    return elem->state;
}

FFIElement get_parent_element(const FFIElement& element) {
    MyElement* elem = reinterpret_cast<MyElement*>(element.ptr);
    FFIElement result;
    result.ptr = elem->parent ? reinterpret_cast<uintptr_t>(elem->parent) : 0;
    return result;
}

bool element_has_class(const FFIElement& element, rust::Str clazz) {
    MyElement* elem = reinterpret_cast<MyElement*>(element.ptr);
    std::string class_name(clazz.data(), clazz.length());
    return std::find(elem->classes.begin(), elem->classes.end(), class_name) 
           != elem->classes.end();
}

// ... implement other callbacks

// Usage
MyElement my_div;
my_div.id = reinterpret_cast<uintptr_t>(&my_div);
my_div.tag_name = "div";
my_div.classes = {"my-class", "active"};
my_div.state = 0; // or include states like HOVER | ACTIVE

FFIElement ffi_elem;
ffi_elem.ptr = my_div.id;

auto result = match_selector("div.my-class", ffi_elem);
if (result.matches) {
    std::cout << "Selector matches!" << std::endl;
}
```

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

### CalcResult
```rust
struct CalcResult {
    value: f32,
    success: bool,
}
```

### SelectorMatchResult
```rust
struct SelectorMatchResult {
    matches: bool,
    error_message: String,
}
```

### FFIElement
```rust
struct FFIElement {
    ptr: usize,  // Opaque pointer to C++ element
}
```

## Building

The FFI is automatically built when you build the stylo crate with the following dependencies:
- `cxx = "1.0"` (runtime dependency)
- `cxx-build = "1.0"` (build dependency)

The build.rs file handles cxx code generation.

## Usage from C++

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
- ✅ Selector parsing
- ✅ Selector matching with FFI elements
- ✅ Element and document state exposure
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
- [ ] Attribute matching for selectors
- [ ] Pseudo-element support in FFI
- [ ] Shadow DOM support in FFI

## Technical Details

### Architecture

The FFI layer is designed to be:
- **Safe**: Uses cxx for type-safe C++/Rust interop
- **Minimal**: Exposes only essential parsing functions
- **Extensible**: Easy to add new functions without breaking changes
- **Efficient**: Zero-copy where possible, minimal allocations
- **Flexible**: C++ can provide custom element implementations

### Selector Matching Architecture

The selector matching system uses a callback-based approach:
1. Rust defines the `FFIElement` struct with an opaque pointer
2. C++ implements callbacks that Rust calls to navigate the DOM tree
3. C++ maintains its own element tree structure
4. Rust wraps `FFIElement` in a type that implements the `Element` trait
5. The selector matching engine uses these callbacks to traverse and query elements

This design allows C++ applications to use Stylo's selector engine without changing their existing DOM structures.

### Error Handling

All functions return result types with success flags and error messages:
- `ParseResult` for stylesheet and media query parsing
- `ParsedCSSValue` for value parsing operations
- `CalcResult` for calc expression evaluation
- `SelectorMatchResult` for selector matching

Errors are descriptive and include context about what failed.

### Limitations

1. **Computed Values**: Full computed value resolution requires an element context and rendering state, which is not yet exposed
2. **Calc Evaluation**: Complex calc() expressions with mixed units require a resolution context
3. **Property-Specific Parsing**: Individual property value parsing is generic; property-specific validation will be added incrementally
4. **Attribute Matching**: Attribute selectors are not yet fully supported in FFI (always return false)
5. **Pseudo-elements**: Pseudo-elements are not supported in FFI selector matching
6. **Shadow DOM**: Shadow DOM features are not exposed through FFI

## Notes

- The current implementation provides robust parsing and basic selector matching facilities
- Error handling returns success/failure status with descriptive messages
- The API is designed to be extended without breaking changes
- Generated C++ code is compatible with C++14 and later
- C++ applications have full control over their element tree structure
- Element and document states are exposed as bitflags for efficient state management
