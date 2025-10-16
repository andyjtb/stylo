/**
 * C++ example demonstrating color parsing with Stylo CSS Parser FFI
 * 
 * This is the C++ equivalent of the Rust color_parser.rs example
 * 
 * To compile and run this example:
 *   cmake --build build
 *   ./build/color_parser
 */

#include <iostream>
#include <string>
#include <iomanip>
#include "stylo/ffi/mod.rs.h"

// Helper function to get color space name
const char* get_color_space_name(ColorSpace cs) {
    switch(cs) {
        case ColorSpace::Srgb: return "sRGB";
        case ColorSpace::Hsl: return "HSL";
        case ColorSpace::Hwb: return "HWB";
        case ColorSpace::Lab: return "Lab";
        case ColorSpace::Lch: return "Lch";
        case ColorSpace::Oklab: return "Oklab";
        case ColorSpace::Oklch: return "Oklch";
        case ColorSpace::SrgbLinear: return "sRGB Linear";
        case ColorSpace::DisplayP3: return "Display P3";
        case ColorSpace::A98Rgb: return "Adobe RGB (1998)";
        case ColorSpace::ProphotoRgb: return "ProPhoto RGB";
        case ColorSpace::Rec2020: return "Rec. 2020";
        case ColorSpace::XyzD50: return "XYZ D50";
        case ColorSpace::XyzD65: return "XYZ D65";
        default: return "Unknown";
    }
}

// Helper function to print color components with appropriate labels
void print_color_components(const ParsedColor& color) {
    const char* space_name = get_color_space_name(color.color_space);
    
    std::cout << "  Color Space: " << space_name << std::endl;
    std::cout << "  Components: (" 
              << std::fixed << std::setprecision(4)
              << color.components.c0 << ", "
              << color.components.c1 << ", "
              << color.components.c2 << ")" << std::endl;
    std::cout << "  Alpha: " << color.alpha << std::endl;
}

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
        std::cout << "✓ Successfully parsed color!" << std::endl;
        print_color_components(result);
    } else {
        std::cout << "✗ Failed to parse color: " << std::string(result.error_message) << std::endl;
    }
    std::cout << std::endl;
    
    // Additional color parsing examples
    std::cout << "=== Additional Color Examples ===" << std::endl;
    std::cout << std::endl;
    
    // Example 1: Named color
    std::cout << "1. Named color (red):" << std::endl;
    auto red_result = parse_color("red");
    if (red_result.success) {
        print_color_components(red_result);
    }
    std::cout << std::endl;
    
    // Example 2: Hex color
    std::cout << "2. Hex color (#ff0000):" << std::endl;
    auto hex_result = parse_color("#ff0000");
    if (hex_result.success) {
        print_color_components(hex_result);
    }
    std::cout << std::endl;
    
    // Example 3: RGB color
    std::cout << "3. RGB color (rgb(255, 0, 0)):" << std::endl;
    auto rgb_result = parse_color("rgb(255, 0, 0)");
    if (rgb_result.success) {
        print_color_components(rgb_result);
    }
    std::cout << std::endl;
    
    // Example 4: RGBA color
    std::cout << "4. RGBA color (rgba(0, 128, 255, 0.5)):" << std::endl;
    auto rgba_result = parse_color("rgba(0, 128, 255, 0.5)");
    if (rgba_result.success) {
        print_color_components(rgba_result);
    }
    std::cout << std::endl;
    
    // Example 5: HSL color
    std::cout << "5. HSL color (hsl(120, 100%, 50%)):" << std::endl;
    auto hsl_result = parse_color("hsl(120, 100%, 50%)");
    if (hsl_result.success) {
        print_color_components(hsl_result);
    }
    std::cout << std::endl;
    
    // Example 6: Lab color
    std::cout << "6. Lab color (lab(50% 20 30)):" << std::endl;
    auto lab_result = parse_color("lab(50% 20 30)");
    if (lab_result.success) {
        print_color_components(lab_result);
    }
    std::cout << std::endl;
    
    // Example 7: Invalid color
    std::cout << "7. Invalid color (notacolor):" << std::endl;
    auto invalid_result = parse_color("notacolor");
    if (!invalid_result.success) {
        std::cout << "  ✓ Correctly rejected: " << std::string(invalid_result.error_message) << std::endl;
    }
    std::cout << std::endl;
    
    std::cout << "=== Example Complete ===" << std::endl;
    
    return 0;
}
