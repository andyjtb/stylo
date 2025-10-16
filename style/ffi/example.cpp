/**
 * Example C++ program demonstrating Stylo CSS Parser FFI usage
 * 
 * To compile this example:
 * 1. Build the stylo library with: cargo build
 * 2. Link against the generated static library
 * 3. Include the generated bridge header
 * 
 * Build command example:
 * g++ -std=c++14 \
 *     -I target/debug/build/stylo-HASH/out/cxxbridge/include \
 *     -L target/debug \
 *     -o example \
 *     example.cpp \
 *     -lstylo_css_parser_ffi \
 *     -lstylo \
 *     -lpthread -ldl
 */

#include <iostream>
#include <string>
#include "stylo/ffi/mod.rs.h"

int main() {
    std::cout << "=== Stylo CSS Parser FFI Demo ===" << std::endl;
    std::cout << std::endl;
    
    // Example 1: Parse a stylesheet
    std::cout << "1. Parsing CSS Stylesheet:" << std::endl;
    auto stylesheet_result = parse_stylesheet(
        "body { color: red; font-size: 16px; margin: 0; }",
        "https://example.com/style.css"
    );
    
    if (stylesheet_result.success) {
        std::cout << "   ✓ Stylesheet parsed successfully!" << std::endl;
    } else {
        std::cout << "   ✗ Error: " << std::string(stylesheet_result.error_message) << std::endl;
    }
    std::cout << std::endl;
    
    // Example 2: Parse media query
    std::cout << "2. Parsing Media Query:" << std::endl;
    auto media_result = parse_media_query("screen and (min-width: 768px)");
    
    if (media_result.success) {
        std::cout << "   ✓ Media query is valid!" << std::endl;
    } else {
        std::cout << "   ✗ Error: " << std::string(media_result.error_message) << std::endl;
    }
    std::cout << std::endl;
    
    // Example 3: Parse CSS value
    std::cout << "3. Parsing CSS Value:" << std::endl;
    auto value_result = parse_css_value("10px", "width");
    
    if (value_result.success) {
        std::cout << "   ✓ Parsed value: " << std::string(value_result.value) << std::endl;
    } else {
        std::cout << "   ✗ Failed to parse value" << std::endl;
    }
    std::cout << std::endl;
    
    // Example 4: Evaluate calc() expression
    std::cout << "4. Evaluating calc() Expression:" << std::endl;
    auto calc_result = evaluate_calc_expression("calc(100)");
    
    if (calc_result.success) {
        std::cout << "   ✓ Calc result: " << calc_result.value << std::endl;
    } else {
        std::cout << "   ✗ Failed to evaluate" << std::endl;
    }
    std::cout << std::endl;
    
    // Example 5: Simple number (fallback behavior)
    std::cout << "5. Evaluating Simple Number (Fallback):" << std::endl;
    auto number_result = evaluate_calc_expression("42.5");
    
    if (number_result.success) {
        std::cout << "   ✓ Number value: " << number_result.value << std::endl;
    } else {
        std::cout << "   ✗ Failed to evaluate" << std::endl;
    }
    std::cout << std::endl;
    
    // Example 6: Get computed value
    std::cout << "6. Getting Computed Value:" << std::endl;
    auto computed = get_computed_value("2em", "font-size", 16.0f);
    
    if (computed.success) {
        std::cout << "   ✓ Computed value: " << std::string(computed.value) << std::endl;
    } else {
        std::cout << "   ✗ Failed to compute" << std::endl;
    }
    std::cout << std::endl;
    
    // Example 7: Invalid URL handling
    std::cout << "7. Error Handling (Invalid URL):" << std::endl;
    auto error_result = parse_stylesheet(
        "body { color: blue; }",
        "not a valid url"
    );
    
    if (!error_result.success) {
        std::cout << "   ✓ Error caught: " << std::string(error_result.error_message) << std::endl;
    }
    std::cout << std::endl;
    
    std::cout << "=== Demo Complete ===" << std::endl;
    
    return 0;
}
