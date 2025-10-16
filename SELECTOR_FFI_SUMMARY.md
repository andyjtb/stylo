# Selector Matching FFI Implementation Summary

This implementation adds selector matching functionality to the Stylo CSS engine, exposing it to C++ through FFI as requested in the issue.

## What Was Implemented

### 1. Selector Parsing API
- `parse_selector(selector: &str) -> ParseResult`
- Parses CSS selector strings and validates them
- Returns success/failure with error messages

### 2. Selector Matching API  
- `match_selector(selector: &str, element: &FFIElement) -> SelectorMatchResult`
- Matches a CSS selector against a C++ element
- Returns whether the selector matches

### 3. Element State Exposure
- **ElementState** exposed as u64 bitflags
  - Contains states like ACTIVE, FOCUS, HOVER, ENABLED, DISABLED, CHECKED, etc.
  - Defined in `stylo_dom` crate
  - Accessible from C++ via `get_element_state()` callback

- **DocumentState** exposed as u64 bitflags
  - Contains states like WINDOW_INACTIVE, RTL_LOCALE, LTR_LOCALE
  - Defined in `stylo_dom` crate
  - Accessible from C++ via `get_document_state()` callback

### 4. C++ Element Integration
The implementation uses a **callback-based approach** where:
- C++ maintains its own element tree structure
- Rust calls C++ functions to navigate and query elements
- Minimal data marshaling across FFI boundary

#### FFIElement Type
```rust
struct FFIElement {
    ptr: usize  // Opaque pointer to C++ element
}
```

#### Required C++ Callbacks
C++ applications must implement these callbacks:

**Navigation:**
- `get_parent_element()` - Get parent element
- `get_prev_sibling_element()` - Get previous sibling
- `get_next_sibling_element()` - Get next sibling  
- `get_first_element_child()` - Get first child

**State:**
- `get_element_state()` - Return element state bitflags
- `get_document_state()` - Return document state bitflags

**Properties:**
- `is_element_null()` - Check if element is null
- `element_has_local_name()` - Check tag name
- `element_has_namespace()` - Check namespace
- `element_has_id()` - Check element ID
- `element_has_class()` - Check element class
- `element_is_link()` - Check if element is a link
- `element_is_root()` - Check if element is root
- `element_is_empty()` - Check if element has no children

### 5. Element Trait Implementation
Created `FFIElementWrapper` that implements `selectors::Element` trait:
- Bridges between FFI callbacks and Rust selector engine
- Handles null elements gracefully
- Converts C++ element state to Rust ElementState
- Supports pseudo-class matching based on state

## Supported Selectors

### ✅ Currently Supported:
- Type selectors (`div`, `span`, `button`)
- ID selectors (`#myid`)
- Class selectors (`.myclass`, `.class1.class2`)
- Combinators (`>`, `+`, `~`, ` `)
- State pseudo-classes (`:hover`, `:active`, `:focus`)
- Link pseudo-classes (`:link`, `:visited`, `:any-link`)
- Form state (`:enabled`, `:disabled`, `:checked`, `:indeterminate`)
- Structural (`:root`, `:empty`)
- UI state (`:valid`, `:invalid`, `:placeholder-shown`)

### ❌ Not Yet Supported in FFI:
- Attribute selectors (`[attr=value]`) - Would need attribute query callback
- Pseudo-elements (`::before`, `::after`) - Not exposed in FFI
- Shadow DOM (`::slotted`, `::part`) - Not exposed in FFI
- Some structural pseudo-classes (`:nth-child`, `:first-child`) - Would need sibling counting
- Custom state (`:--custom`) - Not exposed in FFI

## Files Added/Modified

### Added:
1. **`style/ffi/selector_bridge.h`** - C++ callback declarations
2. **`style/ffi/selector_bridge.cpp`** - Stub implementations of callbacks
3. **`style/ffi/selector_example.cpp`** - Complete C++ usage example

### Modified:
1. **`style/ffi/mod.rs`** - Added selector parsing and matching functions
2. **`style/build.rs`** - Added selector_bridge.cpp to build
3. **`style/ffi/README.md`** - Updated with selector matching documentation
4. **`style/ffi/IMPLEMENTATION.md`** - Added technical implementation details

## How It Works

1. **C++ creates element tree** with custom element types
2. **C++ implements callbacks** to provide element data
3. **C++ calls `match_selector()`** with selector string and element
4. **Rust parses selector** into internal representation
5. **Rust creates FFIElementWrapper** around C++ element pointer
6. **Selector engine matches** by calling FFIElementWrapper methods
7. **FFIElementWrapper methods call C++ callbacks** for element data
8. **Result returned to C++** indicating match status

## Example Usage

```cpp
// C++ element class
class DOMElement {
    uintptr_t id;
    std::string tag_name;
    std::vector<std::string> classes;
    uint64_t state;
    // ... navigation pointers
};

// Implement callbacks
uint64_t get_element_state(const FFIElement& element) {
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    return elem->state;
}

bool element_has_class(const FFIElement& element, rust::Str clazz) {
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    std::string class_name(clazz.data(), clazz.length());
    return std::find(elem->classes.begin(), elem->classes.end(), class_name) 
           != elem->classes.end();
}

// Use selector matching
DOMElement button;
button.tag_name = "button";
button.classes = {"btn", "primary"};
button.state = HOVER;

FFIElement ffi_elem;
ffi_elem.ptr = reinterpret_cast<uintptr_t>(&button);

auto result = match_selector("button.primary:hover", ffi_elem);
// result.matches == true
```

## Design Decisions

### 1. Callback-Based Approach
**Why:** Allows C++ to keep existing DOM structure without marshaling data
- C++ maintains memory management
- No copying of element trees
- Minimal FFI overhead

### 2. Opaque Pointers
**Why:** Simple and efficient
- Just a usize (pointer-sized integer)
- No reference counting needed
- C++ has full control

### 3. State as Bitflags
**Why:** Efficient communication
- Single u64 contains all state
- No string parsing needed
- Matches internal Stylo representation

### 4. Stub Implementations Provided
**Why:** Build works out-of-box
- Developers can test without implementing all callbacks
- Clear template for what needs to be implemented
- Gradual implementation path

## Testing

Added unit tests for:
- Selector parsing (simple and complex selectors)
- Invalid selector handling
- Pseudo-class parsing
- Null element handling

Tests demonstrate the API but cannot fully test matching without C++ element implementation.

## Performance Characteristics

- **Parsing**: Done on each `match_selector()` call (consider caching on C++ side)
- **Matching**: Calls C++ callbacks for each element check
- **Bloom Filter**: Disabled (could be enabled with hash callback)
- **Memory**: No allocations for elements, minimal for selector parsing

## Limitations

1. **No attribute selectors** - Would require additional callback
2. **No pseudo-elements** - Not exposed through FFI
3. **No nth-child** - Would require sibling counting logic
4. **Selector caching** - Not implemented, C++ should cache if needed
5. **Thread safety** - C++ callbacks must be thread-safe if used concurrently

## Future Enhancements

1. Add attribute query callback for attribute selectors
2. Add sibling counting for structural pseudo-classes  
3. Expose parsed selector objects for caching
4. Enable bloom filter with element hash callback
5. Support batch matching for multiple selectors
6. Add invalidation hints for DOM changes

## Conclusion

This implementation successfully:
- ✅ Exposes selector matching to C++
- ✅ Exposes ElementState and DocumentState
- ✅ Allows C++ to provide element tree via callbacks
- ✅ Uses FFI-defined element type (FFIElement)
- ✅ Works with existing Stylo selector engine
- ✅ Provides extensible architecture for future enhancements

The callback-based design allows C++ applications to use Stylo's powerful selector engine while maintaining full control over their DOM tree structure.
