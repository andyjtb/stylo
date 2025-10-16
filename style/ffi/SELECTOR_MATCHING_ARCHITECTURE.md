# Selector Matching FFI Architecture

## Overview

This document outlines the architecture for exposing Stylo's selector matching system to C++ through the FFI.

## Requirements

1. **Element State Exposure**: ElementState and DocumentState bitflags need to be exposed to C++
2. **Element Trait Implementation**: C++ must be able to implement the Element trait through callbacks
3. **Selector Matching**: Parse selectors and match them against C++ elements

## Architecture

### Phase 1: State Bitflags

Expose ElementState and DocumentState as C-compatible bitflags:

```rust
#[cxx::bridge]
mod ffi {
    #[repr(u64)]
    pub enum ElementStateFlags {
        Hover = 0x0001,
        Active = 0x0002,
        Focus = 0x0004,
        // ... other states
    }
    
    #[repr(u64)]
    pub enum DocumentStateFlags {
        WindowInactive = 0x0001,
        // ... other states
    }
}
```

### Phase 2: Element Callbacks

Define callback interface for element operations:

```rust
// C++ implements these callbacks
type ElementParentCallback = fn(element: usize) -> usize;
type ElementPrevSiblingCallback = fn(element: usize) -> usize;
type ElementNextSiblingCallback = fn(element: usize) -> usize;
type ElementHasLocalNameCallback = fn(element: usize, name: &str) -> bool;
type ElementHasNamespaceCallback = fn(element: usize, ns: &str) -> bool;
// ... more callbacks

pub struct ElementCallbacks {
    parent: ElementParentCallback,
    prev_sibling: ElementPrevSiblingCallback,
    next_sibling: ElementNextSiblingCallback,
    has_local_name: ElementHasLocalNameCallback,
    has_namespace: ElementHasNamespaceCallback,
    // ... more callbacks
}
```

### Phase 3: FFI Element Wrapper

Create a wrapper that implements the Element trait using callbacks:

```rust
struct FFIElement {
    handle: usize,  // C++ element pointer cast to usize
    callbacks: &'static ElementCallbacks,
}

impl Element for FFIElement {
    type Impl = SimpleSelector;
    
    fn opaque(&self) -> OpaqueElement {
        // Convert handle to OpaqueElement
    }
    
    fn parent_element(&self) -> Option<Self> {
        let parent_handle = (self.callbacks.parent)(self.handle);
        if parent_handle == 0 {
            None
        } else {
            Some(FFIElement {
                handle: parent_handle,
                callbacks: self.callbacks,
            })
        }
    }
    
    // ... implement all Element trait methods
}
```

### Phase 4: Selector Matching API

```rust
#[cxx::bridge]
mod ffi {
    pub struct SelectorMatchResult {
        pub matches: bool,
        pub error_message: String,
    }
    
    extern "Rust" {
        fn matches_selector(
            element: usize,
            selector: &str,
            callbacks: &ElementCallbacks,
        ) -> SelectorMatchResult;
    }
}

pub fn matches_selector(
    element: usize,
    selector: &str,
    callbacks: &ElementCallbacks,
) -> SelectorMatchResult {
    // Parse selector
    // Create FFIElement from handle and callbacks
    // Run selector matching
    // Return result
}
```

## C++ Usage Example

```cpp
// Inherit from FFI element interface
class MyElement : public FFIElementBase {
public:
    MyElement* parent() override;
    MyElement* prev_sibling() override;
    MyElement* next_sibling() override;
    bool has_local_name(const char* name) override;
    bool has_namespace(const char* ns) override;
    // ... implement other methods
};

// Register callbacks
ElementCallbacks callbacks = {
    .parent = [](usize elem) -> usize {
        return reinterpret_cast<usize>(
            reinterpret_cast<MyElement*>(elem)->parent()
        );
    },
    // ... other callbacks
};

// Match selector
auto result = matches_selector(
    reinterpret_cast<usize>(my_element),
    "div.active:hover",
    &callbacks
);

if (result.matches) {
    // Element matches the selector
}
```

## Challenges

1. **Lifetime Management**: C++ elements must outlive Rust selector matching
2. **Thread Safety**: Callbacks must be thread-safe if matching happens on multiple threads
3. **Performance**: FFI overhead for every element traversal
4. **Namespace Handling**: Need to map C++ strings to Rust atoms efficiently
5. **Attribute Matching**: Complex attribute selectors need efficient C++ callbacks

## Next Steps

1. Define complete ElementState/DocumentState enums from Gecko
2. Implement callback registration system
3. Create FFIElement wrapper implementing Element trait
4. Implement selector parsing and matching
5. Add comprehensive examples and tests
6. Document C++ integration patterns

## Related Files

- `selectors/tree.rs` - Element trait definition
- `style/gecko/wrapper.rs` - Gecko element implementation reference
- `style/matching.rs` - Selector matching implementation
- `style/selector_parser.rs` - Selector parsing

## Notes

This is a complex feature requiring significant development effort. The architecture should be validated with prototypes before full implementation.
