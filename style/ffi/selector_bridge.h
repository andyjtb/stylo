/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/**
 * C++ bridge for selector matching
 * 
 * This header defines the callbacks that C++ must implement to support
 * selector matching through the FFI.
 */

#pragma once

#include <cstdint>
#include "rust/cxx.h"

// Forward declarations
struct FFIElement;

// Callback functions that C++ must implement to support selector matching

/// Get the element state from C++
/// Returns ElementState as u64 bitflags
uint64_t get_element_state(const FFIElement& element);

/// Get the document state from C++
/// Returns DocumentState as u64 bitflags
uint64_t get_document_state(const FFIElement& element);

/// Get parent element
/// Returns a null element if no parent exists
FFIElement get_parent_element(const FFIElement& element);

/// Get previous sibling element
/// Returns a null element if no previous sibling exists
FFIElement get_prev_sibling_element(const FFIElement& element);

/// Get next sibling element
/// Returns a null element if no next sibling exists
FFIElement get_next_sibling_element(const FFIElement& element);

/// Get first child element
/// Returns a null element if no children exist
FFIElement get_first_element_child(const FFIElement& element);

/// Check if element is null/invalid
bool is_element_null(const FFIElement& element);

/// Check if element has a given local name
bool element_has_local_name(const FFIElement& element, rust::Str name);

/// Check if element has a given namespace
bool element_has_namespace(const FFIElement& element, rust::Str ns);

/// Check if element has an id
bool element_has_id(const FFIElement& element, rust::Str id);

/// Check if element has a class
bool element_has_class(const FFIElement& element, rust::Str clazz);

/// Check if element is a link
bool element_is_link(const FFIElement& element);

/// Check if element is root
bool element_is_root(const FFIElement& element);

/// Check if element is empty
bool element_is_empty(const FFIElement& element);
