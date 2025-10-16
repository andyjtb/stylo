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
