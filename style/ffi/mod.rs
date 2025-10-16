/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! C++ FFI for CSS parsing facilities.
//!
//! This module exposes CSS parsing functionality to C++ through the cxx bridge.

use crate::context::QuirksMode;
use crate::media_queries::MediaList;
use crate::parser::ParserContext;
use crate::shared_lock::SharedRwLock;
use crate::stylesheets::{AllowImportRules, Origin, Stylesheet, UrlExtraData};
use servo_arc::Arc;
use style_traits::ParsingMode;

#[cxx::bridge]
mod ffi {
    /// Result of parsing operations
    pub struct ParseResult {
        pub success: bool,
        pub error_message: String,
    }

    /// Result of CSS value parsing
    pub struct ParsedCSSValue {
        pub value: String,
        pub success: bool,
    }

    /// Result of calc expression evaluation
    pub struct CalcResult {
        pub value: f32,
        pub success: bool,
    }

    /// FFI Element representation for selector matching
    /// This is implemented on the C++ side
    #[derive(Debug, Clone)]
    pub struct FFIElement {
        /// Opaque pointer to the C++ element
        pub ptr: usize,
    }

    /// Result of selector matching operation
    pub struct SelectorMatchResult {
        pub matches: bool,
        pub error_message: String,
    }

    unsafe extern "C++" {
        include!("stylo/ffi/selector_bridge.h");

        /// Get the element state from C++
        fn get_element_state(element: &FFIElement) -> u64;
        
        /// Get the document state from C++
        fn get_document_state(element: &FFIElement) -> u64;

        /// Get parent element
        fn get_parent_element(element: &FFIElement) -> FFIElement;

        /// Get previous sibling element
        fn get_prev_sibling_element(element: &FFIElement) -> FFIElement;

        /// Get next sibling element
        fn get_next_sibling_element(element: &FFIElement) -> FFIElement;

        /// Get first child element
        fn get_first_element_child(element: &FFIElement) -> FFIElement;

        /// Check if element is null/invalid
        fn is_element_null(element: &FFIElement) -> bool;

        /// Check if element has a given local name
        fn element_has_local_name(element: &FFIElement, name: &str) -> bool;

        /// Check if element has a given namespace
        fn element_has_namespace(element: &FFIElement, ns: &str) -> bool;

        /// Check if element has an id
        fn element_has_id(element: &FFIElement, id: &str) -> bool;

        /// Check if element has a class
        fn element_has_class(element: &FFIElement, clazz: &str) -> bool;

        /// Check if element is a link
        fn element_is_link(element: &FFIElement) -> bool;

        /// Check if element is root
        fn element_is_root(element: &FFIElement) -> bool;

        /// Check if element is empty
        fn element_is_empty(element: &FFIElement) -> bool;
    }

    extern "Rust" {
        /// Parse a CSS stylesheet from a string
        fn parse_stylesheet(css: &str, base_url: &str) -> ParseResult;

        /// Parse a single CSS value
        fn parse_css_value(value: &str, property_name: &str) -> ParsedCSSValue;

        /// Evaluate a calc() expression
        fn evaluate_calc_expression(expr: &str) -> CalcResult;

        /// Parse and set media query
        fn parse_media_query(query: &str) -> ParseResult;

        /// Get computed value for a CSS property
        fn get_computed_value(
            value: &str,
            property_name: &str,
            base_font_size: f32,
        ) -> ParsedCSSValue;

        /// Parse a CSS selector
        fn parse_selector(selector: &str) -> ParseResult;

        /// Match a selector against an element
        fn match_selector(selector: &str, element: &FFIElement) -> SelectorMatchResult;
    }
}

/// Parse a CSS stylesheet from a string
pub fn parse_stylesheet(css: &str, base_url: &str) -> ffi::ParseResult {
    // Parse the base URL
    let url = match url::Url::parse(base_url) {
        Ok(u) => u,
        Err(e) => {
            return ffi::ParseResult {
                success: false,
                error_message: format!("Invalid base URL: {:?}", e),
            };
        }
    };

    let url_data = UrlExtraData::from(url);
    let shared_lock = SharedRwLock::new();

    let media = Arc::new(shared_lock.wrap(MediaList::empty()));

    let _stylesheet = Stylesheet::from_str(
        css,
        url_data,
        Origin::Author,
        media,
        shared_lock,
        None, // No stylesheet loader
        None, // No error reporter
        QuirksMode::NoQuirks,
        AllowImportRules::Yes,
    );

    // Stylesheet parsing always succeeds and records errors internally
    ffi::ParseResult {
        success: true,
        error_message: String::new(),
    }
}

/// Parse a single CSS value
pub fn parse_css_value(value: &str, _property_name: &str) -> ffi::ParsedCSSValue {
    use cssparser::{Parser, ParserInput};

    // Parse the value using cssparser
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);

    let url = url::Url::parse("about:blank").unwrap();
    let url_data = UrlExtraData::from(url);

    let _context = ParserContext::new(
        Origin::Author,
        &url_data,
        None,
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        Default::default(),
        None,
        None,
    );

    // Try parsing as a generic CSS value (length, color, etc.)
    // For now, we validate it's parseable and return the original
    match parser.expect_no_error_token() {
        Ok(_) => ffi::ParsedCSSValue {
            value: value.to_string(),
            success: true,
        },
        Err(_) => ffi::ParsedCSSValue {
            value: String::new(),
            success: false,
        },
    }
}

/// Evaluate a calc() expression
pub fn evaluate_calc_expression(expr: &str) -> ffi::CalcResult {
    use crate::values::generics::calc::CalcUnits;
    use crate::values::specified::calc::{AllowParse, CalcNode, MathFunction};
    use cssparser::{Parser, ParserInput};

    // Try to parse as a calc expression
    let url = url::Url::parse("about:blank").unwrap();
    let url_data = UrlExtraData::from(url);
    let mut input = ParserInput::new(expr);
    let mut parser = Parser::new(&mut input);

    let context = ParserContext::new(
        Origin::Author,
        &url_data,
        None,
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        Default::default(),
        None,
        None,
    );

    // Try parsing as calc() function
    if let Ok(calc_node) = parser.try_parse(|p| {
        p.expect_function_matching("calc")?;
        p.parse_nested_block(|p| {
            CalcNode::parse(
                &context,
                p,
                MathFunction::Calc,
                AllowParse::new(CalcUnits::all()),
            )
        })
    }) {
        // For simple numeric results, try to extract the value
        // This is a simplified evaluation - full evaluation requires context
        if let Some(numeric_value) = try_extract_numeric_value(&calc_node) {
            return ffi::CalcResult {
                value: numeric_value,
                success: true,
            };
        }
    }

    // Fallback: try parsing as simple numeric value
    let trimmed = expr.trim();
    if let Ok(val) = trimmed.parse::<f32>() {
        return ffi::CalcResult {
            value: val,
            success: true,
        };
    }

    ffi::CalcResult {
        value: 0.0,
        success: false,
    }
}

/// Try to extract a numeric value from a calc node
/// This is a simplified version - full evaluation requires context
fn try_extract_numeric_value(node: &crate::values::specified::calc::CalcNode) -> Option<f32> {
    use crate::values::specified::calc::{CalcNode, Leaf};

    match node {
        CalcNode::Leaf(Leaf::Number(n)) => Some(*n),
        CalcNode::Leaf(Leaf::Percentage(p)) => Some(*p),
        _ => None, // More complex expressions need context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stylesheet() {
        let result = parse_stylesheet(
            "body { color: red; font-size: 16px; }",
            "https://example.com/style.css",
        );
        assert!(result.success, "Stylesheet parsing should succeed");
        assert!(result.error_message.is_empty());
    }

    #[test]
    fn test_parse_stylesheet_invalid_url() {
        let result = parse_stylesheet("body { color: red; }", "not a url");
        assert!(!result.success, "Should fail with invalid URL");
        assert!(!result.error_message.is_empty());
    }

    #[test]
    fn test_parse_media_query() {
        let result = parse_media_query("(min-width: 768px)");
        assert!(result.success, "Media query parsing should succeed");
        assert!(result.error_message.is_empty());
    }

    #[test]
    fn test_parse_media_query_screen() {
        let result = parse_media_query("screen and (min-width: 768px)");
        assert!(result.success, "Media query with type should succeed");
    }

    #[test]
    fn test_parse_css_value() {
        let result = parse_css_value("10px", "width");
        assert!(result.success, "Should parse simple value");
    }

    #[test]
    fn test_evaluate_calc_simple_number() {
        let result = evaluate_calc_expression("42");
        assert!(result.success, "Should parse simple number");
        assert_eq!(result.value, 42.0);
    }

    #[test]
    fn test_evaluate_calc_function() {
        let result = evaluate_calc_expression("calc(5)");
        assert!(result.success, "Should parse calc with number");
        assert_eq!(result.value, 5.0);
    }

    #[test]
    fn test_get_computed_value() {
        let result = get_computed_value("2em", "font-size", 16.0);
        assert!(result.success);
    }

    #[test]
    fn test_parse_selector_simple() {
        let result = parse_selector("div");
        assert!(result.success, "Should parse simple selector");
        assert!(result.error_message.is_empty());
    }

    #[test]
    fn test_parse_selector_complex() {
        let result = parse_selector("div.my-class:hover > span#id");
        assert!(result.success, "Should parse complex selector");
        assert!(result.error_message.is_empty());
    }

    #[test]
    fn test_parse_selector_invalid() {
        let result = parse_selector(">>>invalid");
        assert!(!result.success, "Should fail on invalid selector");
        assert!(!result.error_message.is_empty());
    }

    #[test]
    fn test_match_selector_null_element() {
        let element = ffi::FFIElement { ptr: 0 };
        let result = match_selector("div", &element);
        // With a null element, is_element_null returns true,
        // so most operations should return false/no match
        assert!(!result.matches);
    }

    #[test]
    fn test_parse_selector_pseudo_classes() {
        let result = parse_selector("a:link");
        assert!(result.success, "Should parse :link pseudo-class");
        
        let result = parse_selector("input:checked");
        assert!(result.success, "Should parse :checked pseudo-class");
        
        let result = parse_selector("div:hover");
        assert!(result.success, "Should parse :hover pseudo-class");
    }
}

/// Parse and validate a media query
pub fn parse_media_query(query: &str) -> ffi::ParseResult {
    use crate::media_queries::MediaQuery;
    use cssparser::{Parser, ParserInput};

    // Use a dummy URL for parsing media queries
    let url = url::Url::parse("about:blank").unwrap();
    let url_data = UrlExtraData::from(url);
    let mut input = ParserInput::new(query);
    let mut parser = Parser::new(&mut input);

    let context = ParserContext::new(
        Origin::Author,
        &url_data,
        None,
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        Default::default(),
        None,
        None,
    );

    match MediaQuery::parse(&context, &mut parser) {
        Ok(_mq) => ffi::ParseResult {
            success: true,
            error_message: String::new(),
        },
        Err(e) => ffi::ParseResult {
            success: false,
            error_message: format!("Failed to parse media query: {:?}", e),
        },
    }
}

/// Get computed value for a CSS property
pub fn get_computed_value(
    value: &str,
    _property_name: &str,
    _base_font_size: f32,
) -> ffi::ParsedCSSValue {
    // This would need full implementation with a proper ComputedContext
    // For now, return the input value
    ffi::ParsedCSSValue {
        value: value.to_string(),
        success: !value.is_empty(),
    }
}

/// Parse a CSS selector
pub fn parse_selector(selector: &str) -> ffi::ParseResult {
    use crate::selector_parser::SelectorParser;
    
    let url = url::Url::parse("about:blank").unwrap();
    let url_data = UrlExtraData::from(url);
    
    match SelectorParser::parse_author_origin_no_namespace(selector, &url_data) {
        Ok(_selector_list) => ffi::ParseResult {
            success: true,
            error_message: String::new(),
        },
        Err(e) => ffi::ParseResult {
            success: false,
            error_message: format!("Failed to parse selector: {:?}", e),
        },
    }
}

/// Match a selector against an element
pub fn match_selector(selector: &str, element: &ffi::FFIElement) -> ffi::SelectorMatchResult {
    use crate::selector_parser::SelectorParser;
    use selectors::matching::{matches_selector, MatchingContext, MatchingMode, NeedsSelectorFlags, MatchingForInvalidation};
    use selectors::context::{QuirksMode as SelectorQuirksMode, SelectorCaches};
    
    // Parse the selector
    let url = url::Url::parse("about:blank").unwrap();
    let url_data = UrlExtraData::from(url);
    
    let selector_list = match SelectorParser::parse_author_origin_no_namespace(selector, &url_data) {
        Ok(list) => list,
        Err(e) => {
            return ffi::SelectorMatchResult {
                matches: false,
                error_message: format!("Failed to parse selector: {:?}", e),
            };
        }
    };

    // Create FFI element wrapper
    let ffi_elem = FFIElementWrapper(element.clone());
    
    // Create matching context
    let mut caches = SelectorCaches::default();
    let mut context = MatchingContext::new(
        MatchingMode::Normal,
        None, // bloom filter
        &mut caches,
        SelectorQuirksMode::NoQuirks,
        NeedsSelectorFlags::No,
        MatchingForInvalidation::No,
    );

    // Check if any selector in the list matches
    for selector in selector_list.slice().iter() {
        if matches_selector(selector, 0, None, &ffi_elem, &mut context) {
            return ffi::SelectorMatchResult {
                matches: true,
                error_message: String::new(),
            };
        }
    }

    ffi::SelectorMatchResult {
        matches: false,
        error_message: String::new(),
    }
}

/// Wrapper around FFIElement that implements the Element trait
#[derive(Clone, Debug)]
struct FFIElementWrapper(ffi::FFIElement);

impl selectors::Element for FFIElementWrapper {
    type Impl = crate::selector_parser::SelectorImpl;

    fn opaque(&self) -> selectors::OpaqueElement {
        selectors::OpaqueElement::from_non_null_ptr(
            std::ptr::NonNull::new(self.0.ptr as *mut ()).expect("Element pointer should not be null")
        )
    }

    fn parent_element(&self) -> Option<Self> {
        let parent = ffi::get_parent_element(&self.0);
        if ffi::is_element_null(&parent) {
            None
        } else {
            Some(FFIElementWrapper(parent))
        }
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false // Not supported in FFI for now
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None // Not supported in FFI for now
    }

    fn is_pseudo_element(&self) -> bool {
        false // Not supported in FFI for now
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        let sibling = ffi::get_prev_sibling_element(&self.0);
        if ffi::is_element_null(&sibling) {
            None
        } else {
            Some(FFIElementWrapper(sibling))
        }
    }

    fn next_sibling_element(&self) -> Option<Self> {
        let sibling = ffi::get_next_sibling_element(&self.0);
        if ffi::is_element_null(&sibling) {
            None
        } else {
            Some(FFIElementWrapper(sibling))
        }
    }

    fn first_element_child(&self) -> Option<Self> {
        let child = ffi::get_first_element_child(&self.0);
        if ffi::is_element_null(&child) {
            None
        } else {
            Some(FFIElementWrapper(child))
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        // For FFI, we'll assume HTML context
        true
    }

    fn has_local_name(&self, local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName) -> bool {
        ffi::element_has_local_name(&self.0, local_name)
    }

    fn has_namespace(&self, ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        ffi::element_has_namespace(&self.0, ns)
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.0.ptr == other.0.ptr
    }

    fn attr_matches(
        &self,
        _ns: &selectors::attr::NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        _local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        _operation: &selectors::attr::AttrSelectorOperation<&<Self::Impl as selectors::SelectorImpl>::AttrValue>,
    ) -> bool {
        // Attribute matching would need more FFI callbacks
        false
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        _context: &mut selectors::matching::MatchingContext<Self::Impl>,
    ) -> bool {
        use crate::selector_parser::NonTSPseudoClass;
        use dom::ElementState;

        let state = ElementState::from_bits_truncate(ffi::get_element_state(&self.0));
        
        // Match against element state
        match pc {
            NonTSPseudoClass::Active => state.contains(ElementState::ACTIVE),
            NonTSPseudoClass::Focus => state.contains(ElementState::FOCUS),
            NonTSPseudoClass::Hover => state.contains(ElementState::HOVER),
            NonTSPseudoClass::Enabled => state.contains(ElementState::ENABLED),
            NonTSPseudoClass::Disabled => state.contains(ElementState::DISABLED),
            NonTSPseudoClass::Checked => state.contains(ElementState::CHECKED),
            NonTSPseudoClass::Indeterminate => state.contains(ElementState::INDETERMINATE),
            NonTSPseudoClass::PlaceholderShown => state.contains(ElementState::PLACEHOLDER_SHOWN),
            NonTSPseudoClass::Target => state.contains(ElementState::URLTARGET),
            NonTSPseudoClass::Visited => state.contains(ElementState::VISITED),
            NonTSPseudoClass::Link => state.contains(ElementState::UNVISITED),
            NonTSPseudoClass::AnyLink => state.intersects(ElementState::VISITED_OR_UNVISITED),
            _ => false, // Other pseudo-classes not yet implemented for FFI
        }
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        _context: &mut selectors::matching::MatchingContext<Self::Impl>,
    ) -> bool {
        false // Pseudo-elements not supported in FFI for now
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {
        // No-op for FFI elements
    }

    fn is_link(&self) -> bool {
        ffi::element_is_link(&self.0)
    }

    fn is_html_slot_element(&self) -> bool {
        false // Not supported in FFI for now
    }

    fn has_id(
        &self,
        id: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        _case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        ffi::element_has_id(&self.0, id)
    }

    fn has_class(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        _case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        ffi::element_has_class(&self.0, name)
    }

    fn has_custom_state(&self, _name: &<Self::Impl as selectors::SelectorImpl>::Identifier) -> bool {
        false // Not supported in FFI for now
    }

    fn imported_part(
        &self,
        _name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        None // Not supported in FFI for now
    }

    fn is_part(&self, _name: &<Self::Impl as selectors::SelectorImpl>::Identifier) -> bool {
        false // Not supported in FFI for now
    }

    fn is_empty(&self) -> bool {
        ffi::element_is_empty(&self.0)
    }

    fn is_root(&self) -> bool {
        ffi::element_is_root(&self.0)
    }

    fn add_element_unique_hashes(&self, _filter: &mut selectors::bloom::BloomFilter) -> bool {
        false // Bloom filter not used for FFI
    }
}
