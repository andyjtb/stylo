# Stylo FFI Implementation Details

This document provides technical details about the Stylo FFI implementation.

## Architecture Overview

The FFI layer uses [cxx](https://cxx.rs/) to provide safe, type-checked interop between Rust and C++.

### Build Process

1. `build.rs` runs during compilation
2. Generates properties code from Mako templates
3. Invokes `cxx_build::bridge()` to generate C++/Rust bridge code
4. Compiles C++ implementation files (`css_parser_bridge.cpp`, `selector_bridge.cpp`)
5. Links everything together

### Generated Files

cxx generates the following during build:
- `target/<profile>/cxxbridge/stylo/ffi/mod.rs.h` - C++ header with Rust function declarations
- `target/<profile>/cxxbridge/sources/stylo/ffi/mod.rs.cc` - C++ bridge implementation
- Rust FFI bindings for C++ functions

## CSS Parser FFI

### Stylesheet Parsing

```rust
pub fn parse_stylesheet(css: &str, base_url: &str) -> ParseResult
```

Implementation:
1. Parses base URL using `url::Url::parse()`
2. Creates `UrlExtraData` for the stylesheet
3. Uses `Stylesheet::from_str()` to parse CSS
4. Returns success/failure with error message

### Media Query Parsing

```rust
pub fn parse_media_query(query: &str) -> ParseResult
```

Implementation:
1. Creates a `ParserContext` with default settings
2. Uses `MediaQuery::parse()` from the media_queries module
3. Returns validation result

### Calc Expression Evaluation

```rust
pub fn evaluate_calc_expression(expr: &str) -> CalcResult
```

Implementation:
1. Attempts to parse as `calc()` function
2. Uses `CalcNode::parse()` with all calc units allowed
3. Tries to extract numeric value from simple expressions
4. Falls back to parsing as plain number

## Selector Matching FFI

The selector matching system provides a bridge between Rust's selector engine and C++ element trees.

### Design Philosophy

Rather than trying to marshal C++ elements into Rust, we use a callback-based approach:
- C++ maintains its own element tree
- Rust calls C++ functions to navigate and query elements
- Minimal data copying; pointers used for identity

### FFIElement Type

```rust
#[derive(Debug, Clone)]
pub struct FFIElement {
    pub ptr: usize,
}
```

- Opaque pointer to C++ element
- Copied by value (cheap, just a usize)
- C++ is responsible for memory management
- Null element represented by `ptr = 0`

### Element Trait Implementation

`FFIElementWrapper` implements the `selectors::Element` trait:

```rust
struct FFIElementWrapper(FFIElement);

impl selectors::Element for FFIElementWrapper {
    type Impl = crate::selector_parser::SelectorImpl;
    // ... implementation
}
```

Key trait methods:
- `parent_element()` - Calls C++ `get_parent_element()`
- `prev_sibling_element()` - Calls C++ `get_prev_sibling_element()`
- `next_sibling_element()` - Calls C++ `get_next_sibling_element()`
- `first_element_child()` - Calls C++ `get_first_element_child()`
- `has_local_name()` - Calls C++ `element_has_local_name()`
- `has_id()` - Calls C++ `element_has_id()`
- `has_class()` - Calls C++ `element_has_class()`
- `match_non_ts_pseudo_class()` - Uses element state from C++

### Selector Matching Flow

1. C++ calls `match_selector(selector, element)`
2. Rust parses selector string into `SelectorList`
3. Creates `FFIElementWrapper` around `FFIElement`
4. Creates `MatchingContext` with no bloom filter
5. For each selector in list:
   - Calls `matches_selector()` from selectors crate
   - Selector engine calls trait methods on `FFIElementWrapper`
   - These methods call back to C++ functions
   - C++ provides element data
6. Returns `SelectorMatchResult` with match status

### State Management

Element and document states are exposed as u64 bitflags:

#### ElementState (from `dom` crate)
- Represented as `u64` bitmask
- Flags include: ACTIVE, FOCUS, HOVER, ENABLED, DISABLED, CHECKED, etc.
- C++ returns state via `get_element_state()`
- Rust converts to `ElementState` with `from_bits_truncate()`

#### DocumentState (from `dom` crate)
- Represented as `u64` bitmask  
- Flags include: WINDOW_INACTIVE, RTL_LOCALE, LTR_LOCALE
- C++ returns state via `get_document_state()`

Example state usage in C++:
```cpp
const uint64_t HOVER = 1 << 2;
const uint64_t ACTIVE = 1 << 0;

DOMElement elem;
elem.state = HOVER | ACTIVE;  // Element is being hovered and active
```

### Required C++ Callbacks

C++ must implement these functions:

**Navigation:**
- `FFIElement get_parent_element(const FFIElement&)`
- `FFIElement get_prev_sibling_element(const FFIElement&)`
- `FFIElement get_next_sibling_element(const FFIElement&)`
- `FFIElement get_first_element_child(const FFIElement&)`

**State:**
- `uint64_t get_element_state(const FFIElement&)`
- `uint64_t get_document_state(const FFIElement&)`

**Properties:**
- `bool is_element_null(const FFIElement&)`
- `bool element_has_local_name(const FFIElement&, rust::Str)`
- `bool element_has_namespace(const FFIElement&, rust::Str)`
- `bool element_has_id(const FFIElement&, rust::Str)`
- `bool element_has_class(const FFIElement&, rust::Str)`
- `bool element_is_link(const FFIElement&)`
- `bool element_is_root(const FFIElement&)`
- `bool element_is_empty(const FFIElement&)`

### Selector Support

Currently supported:
- ✅ Type selectors (`div`, `span`)
- ✅ ID selectors (`#myid`)
- ✅ Class selectors (`.myclass`)
- ✅ Combinators (`>`, `+`, `~`, ` `)
- ✅ State pseudo-classes (`:hover`, `:active`, `:focus`, etc.)
- ✅ Link pseudo-classes (`:link`, `:visited`, `:any-link`)
- ✅ Form state pseudo-classes (`:enabled`, `:disabled`, `:checked`)
- ✅ Root pseudo-class (`:root`)
- ✅ Empty pseudo-class (`:empty`)

Not yet supported in FFI:
- ❌ Attribute selectors (`[attr=value]`)
- ❌ Pseudo-elements (`::before`, `::after`)
- ❌ Shadow DOM pseudo-elements (`::slotted`, `::part`)
- ❌ Custom state (`:--custom`)
- ❌ Some structural pseudo-classes (`:nth-child`, `:first-child`, etc.)

The unsupported features return false or are not available due to limitations in the current FFI implementation.

## Performance Considerations

### Zero-Copy Design
- Strings passed as `rust::Str` (string views, no copy)
- Elements passed as pointers (usize)
- State passed as primitive u64

### Bloom Filter
- Currently disabled for FFI (`bloom_filter: None`)
- Could be enabled if C++ provides element hashes
- Would improve performance for deep selector matching

### Caching
- No caching currently implemented
- C++ is responsible for any element caching
- Selector parsing is done on each `match_selector()` call
  - Consider caching parsed selectors on C++ side for repeated matching

## Thread Safety

- FFI functions are not explicitly marked as thread-safe
- Element pointers are passed as raw usize values
- C++ is responsible for:
  - Element lifetime management
  - Thread synchronization if elements are accessed from multiple threads
- Rust selector matching is thread-safe if element callbacks are thread-safe

## Memory Management

- Rust does not manage element memory
- C++ owns all element objects
- FFI only stores element pointers (usize)
- C++ must ensure elements live long enough for matching operation
- No reference counting or garbage collection across FFI boundary

## Error Handling

All public FFI functions return result types:
- Parse errors include descriptive messages
- Invalid selectors are caught during parsing
- Null elements are handled gracefully (return `ptr = 0`)
- C++ callbacks should not throw exceptions (undefined behavior)

## Future Improvements

1. **Attribute Matching**: Implement attribute selectors
   - Requires additional C++ callback for attribute queries
   
2. **Structural Pseudo-classes**: Support :nth-child, :first-child, etc.
   - Requires sibling counting callbacks
   
3. **Selector Caching**: Cache parsed selectors
   - Expose selector list type to C++
   - Allow reusing parsed selectors
   
4. **Bloom Filter**: Enable for performance
   - Add element hash callback
   - Implement bloom filter population
   
5. **Batch Matching**: Match multiple selectors at once
   - More efficient for style resolution
   
6. **Incremental Matching**: Support invalidation
   - Track which selectors might match after DOM changes
