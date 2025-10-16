/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/**
 * C++ implementation of selector bridge callbacks
 * 
 * This provides stub implementations that C++ applications can override
 * to provide their own element tree.
 */

#include "stylo/ffi/mod.rs.h"
#include "stylo/ffi/selector_bridge.h"

// Stub implementations - real applications should override these

uint64_t get_element_state(const FFIElement& element) {
    // Return no state by default
    // Real implementation should check element and return appropriate state flags
    return 0;
}

uint64_t get_document_state(const FFIElement& element) {
    // Return no state by default
    return 0;
}

FFIElement get_parent_element(const FFIElement& element) {
    // Return null element by default
    FFIElement result;
    result.ptr = 0;
    return result;
}

FFIElement get_prev_sibling_element(const FFIElement& element) {
    // Return null element by default
    FFIElement result;
    result.ptr = 0;
    return result;
}

FFIElement get_next_sibling_element(const FFIElement& element) {
    // Return null element by default
    FFIElement result;
    result.ptr = 0;
    return result;
}

FFIElement get_first_element_child(const FFIElement& element) {
    // Return null element by default
    FFIElement result;
    result.ptr = 0;
    return result;
}

bool is_element_null(const FFIElement& element) {
    return element.ptr == 0;
}

bool element_has_local_name(const FFIElement& element, rust::Str name) {
    // Stub: always return false
    return false;
}

bool element_has_namespace(const FFIElement& element, rust::Str ns) {
    // Stub: default to empty namespace
    return ns.empty();
}

bool element_has_id(const FFIElement& element, rust::Str id) {
    // Stub: always return false
    return false;
}

bool element_has_class(const FFIElement& element, rust::Str clazz) {
    // Stub: always return false
    return false;
}

bool element_is_link(const FFIElement& element) {
    // Stub: always return false
    return false;
}

bool element_is_root(const FFIElement& element) {
    // Stub: always return false
    return false;
}

bool element_is_empty(const FFIElement& element) {
    // Stub: always return false
    return false;
}
