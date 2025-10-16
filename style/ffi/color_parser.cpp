/**
 * C++ example demonstrating color parsing with Stylo CSS Parser FFI
 * 
 * This is the C++ equivalent of the Rust color_parser.rs example
 * 
 * To compile and run this example:
 *   make color_parser
 *   make run_color_parser
 */

#include <iostream>
#include <string>
#include "stylo/ffi/mod.rs.h"

int main() {
    std::cout << "=== Stylo Color Parser Example (C++) ===" << std::endl;
    std::cout << std::endl;
    
    // The color string we want to parse (same as Rust example)
    std::string color_str = "hsla(-300, 100%, 37.5%, -3)";
    
    std::cout << "Parsing color: " << color_str << std::endl;
    std::cout << std::endl;
    
    // Parse the color
    auto result = parse_color(color_str);
    
    if (result.success) {
        std::cout << "Successfully parsed color: " << std::string(result.value) << std::endl;
    } else {
        std::cout << "Failed to parse color: " << std::string(result.value) << std::endl;
    }
    std::cout << std::endl;
    
    // Additional color parsing examples
    std::cout << "=== Additional Color Examples ===" << std::endl;
    std::cout << std::endl;
    
    // Example 1: Named color
    std::cout << "1. Named color (red):" << std::endl;
    auto red_result = parse_color("red");
    if (red_result.success) {
        std::cout << "   ✓ Parsed: " << std::string(red_result.value) << std::endl;
    }
    std::cout << std::endl;
    
    // Example 2: Hex color
    std::cout << "2. Hex color (#ff0000):" << std::endl;
    auto hex_result = parse_color("#ff0000");
    if (hex_result.success) {
        std::cout << "   ✓ Parsed: " << std::string(hex_result.value) << std::endl;
    }
    std::cout << std::endl;
    
    // Example 3: RGB color
    std::cout << "3. RGB color (rgb(255, 0, 0)):" << std::endl;
    auto rgb_result = parse_color("rgb(255, 0, 0)");
    if (rgb_result.success) {
        std::cout << "   ✓ Parsed: " << std::string(rgb_result.value) << std::endl;
    }
    std::cout << std::endl;
    
    // Example 4: RGBA color
    std::cout << "4. RGBA color (rgba(0, 128, 255, 0.5)):" << std::endl;
    auto rgba_result = parse_color("rgba(0, 128, 255, 0.5)");
    if (rgba_result.success) {
        std::cout << "   ✓ Parsed: " << std::string(rgba_result.value) << std::endl;
    }
    std::cout << std::endl;
    
    // Example 5: HSL color
    std::cout << "5. HSL color (hsl(120, 100%, 50%)):" << std::endl;
    auto hsl_result = parse_color("hsl(120, 100%, 50%)");
    if (hsl_result.success) {
        std::cout << "   ✓ Parsed: " << std::string(hsl_result.value) << std::endl;
    }
    std::cout << std::endl;
    
    // Example 6: Invalid color
    std::cout << "6. Invalid color (notacolor):" << std::endl;
    auto invalid_result = parse_color("notacolor");
    if (!invalid_result.success) {
        std::cout << "   ✓ Correctly rejected: " << std::string(invalid_result.value) << std::endl;
    }
    std::cout << std::endl;
    
    std::cout << "=== Example Complete ===" << std::endl;
    
    return 0;
}
