/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/**
 * Example C++ usage of Stylo selector matching FFI
 * 
 * This demonstrates how to implement the required callbacks to enable
 * selector matching with C++ element trees.
 */

#include "stylo/ffi/mod.rs.h"
#include <iostream>
#include <vector>
#include <string>
#include <algorithm>

// Example DOM element class
class DOMElement {
public:
    uintptr_t id;
    std::string tag_name;
    std::string element_id;
    std::vector<std::string> classes;
    uint64_t state;
    DOMElement* parent;
    DOMElement* prev_sibling;
    DOMElement* next_sibling;
    DOMElement* first_child;
    
    DOMElement(const std::string& tag)
        : id(reinterpret_cast<uintptr_t>(this)),
          tag_name(tag),
          state(0),
          parent(nullptr),
          prev_sibling(nullptr),
          next_sibling(nullptr),
          first_child(nullptr) {}
};

// ElementState flags (from dom crate)
const uint64_t HOVER = 1 << 2;
const uint64_t ACTIVE = 1 << 0;
const uint64_t FOCUS = 1 << 1;
const uint64_t DISABLED = 1 << 4;
const uint64_t CHECKED = 1 << 5;

// Implement the required callbacks

uint64_t get_element_state(const FFIElement& element) {
    if (element.ptr == 0) return 0;
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    return elem->state;
}

uint64_t get_document_state(const FFIElement& /* element */) {
    // Return document state (e.g., window active, RTL/LTR locale)
    return 0;
}

FFIElement get_parent_element(const FFIElement& element) {
    if (element.ptr == 0) return FFIElement{0};
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    FFIElement result;
    result.ptr = elem->parent ? reinterpret_cast<uintptr_t>(elem->parent) : 0;
    return result;
}

FFIElement get_prev_sibling_element(const FFIElement& element) {
    if (element.ptr == 0) return FFIElement{0};
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    FFIElement result;
    result.ptr = elem->prev_sibling ? reinterpret_cast<uintptr_t>(elem->prev_sibling) : 0;
    return result;
}

FFIElement get_next_sibling_element(const FFIElement& element) {
    if (element.ptr == 0) return FFIElement{0};
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    FFIElement result;
    result.ptr = elem->next_sibling ? reinterpret_cast<uintptr_t>(elem->next_sibling) : 0;
    return result;
}

FFIElement get_first_element_child(const FFIElement& element) {
    if (element.ptr == 0) return FFIElement{0};
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    FFIElement result;
    result.ptr = elem->first_child ? reinterpret_cast<uintptr_t>(elem->first_child) : 0;
    return result;
}

bool is_element_null(const FFIElement& element) {
    return element.ptr == 0;
}

bool element_has_local_name(const FFIElement& element, rust::Str name) {
    if (element.ptr == 0) return false;
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    std::string tag(name.data(), name.length());
    return elem->tag_name == tag;
}

bool element_has_namespace(const FFIElement& /* element */, rust::Str ns) {
    // For HTML, default to empty namespace
    return ns.empty();
}

bool element_has_id(const FFIElement& element, rust::Str id) {
    if (element.ptr == 0) return false;
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    std::string id_str(id.data(), id.length());
    return elem->element_id == id_str;
}

bool element_has_class(const FFIElement& element, rust::Str clazz) {
    if (element.ptr == 0) return false;
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    std::string class_name(clazz.data(), clazz.length());
    return std::find(elem->classes.begin(), elem->classes.end(), class_name) 
           != elem->classes.end();
}

bool element_is_link(const FFIElement& element) {
    if (element.ptr == 0) return false;
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    return elem->tag_name == "a";
}

bool element_is_root(const FFIElement& element) {
    if (element.ptr == 0) return false;
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    return elem->parent == nullptr;
}

bool element_is_empty(const FFIElement& element) {
    if (element.ptr == 0) return true;
    DOMElement* elem = reinterpret_cast<DOMElement*>(element.ptr);
    return elem->first_child == nullptr;
}

// Example usage
int main() {
    // Create a simple DOM tree:
    // <div id="root" class="container">
    //   <button class="btn primary" [hover state]>Click me</button>
    // </div>
    
    DOMElement root("div");
    root.element_id = "root";
    root.classes.push_back("container");
    
    DOMElement button("button");
    button.element_id = "";
    button.classes.push_back("btn");
    button.classes.push_back("primary");
    button.state = HOVER;  // Simulating hover state
    button.parent = &root;
    
    root.first_child = &button;
    
    // Convert to FFI elements
    FFIElement ffi_root;
    ffi_root.ptr = root.id;
    
    FFIElement ffi_button;
    ffi_button.ptr = button.id;
    
    // Test selector matching
    std::cout << "Testing selector matching:" << std::endl;
    
    // Test 1: Simple tag selector
    auto result1 = match_selector("div", ffi_root);
    std::cout << "  div matches root: " << (result1.matches ? "yes" : "no") << std::endl;
    
    // Test 2: ID selector
    auto result2 = match_selector("#root", ffi_root);
    std::cout << "  #root matches root: " << (result2.matches ? "yes" : "no") << std::endl;
    
    // Test 3: Class selector
    auto result3 = match_selector(".container", ffi_root);
    std::cout << "  .container matches root: " << (result3.matches ? "yes" : "no") << std::endl;
    
    // Test 4: Multiple class selector
    auto result4 = match_selector(".btn.primary", ffi_button);
    std::cout << "  .btn.primary matches button: " << (result4.matches ? "yes" : "no") << std::endl;
    
    // Test 5: Pseudo-class selector
    auto result5 = match_selector("button:hover", ffi_button);
    std::cout << "  button:hover matches button: " << (result5.matches ? "yes" : "no") << std::endl;
    
    // Test 6: Complex selector (won't match - wrong tag)
    auto result6 = match_selector("span.primary", ffi_button);
    std::cout << "  span.primary matches button: " << (result6.matches ? "yes" : "no") << std::endl;
    
    // Test selector parsing
    std::cout << "\nTesting selector parsing:" << std::endl;
    auto parse1 = parse_selector("div > .my-class:hover");
    std::cout << "  'div > .my-class:hover' is valid: " << (parse1.success ? "yes" : "no") << std::endl;
    
    auto parse2 = parse_selector(">>>invalid");
    std::cout << "  '>>>invalid' is valid: " << (parse2.success ? "yes" : "no") << std::endl;
    if (!parse2.success) {
        std::cout << "    Error: " << parse2.error_message << std::endl;
    }
    
    return 0;
}
