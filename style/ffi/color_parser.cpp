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

// Helper function to extract RGBA from nscolor uint32
struct RGBA {
    uint8_t r, g, b, a;
};

RGBA nscolor_to_rgba(uint32_t nscolor) {
    return {
        static_cast<uint8_t>(nscolor & 0xFF),
        static_cast<uint8_t>((nscolor >> 8) & 0xFF),
        static_cast<uint8_t>((nscolor >> 16) & 0xFF),
        static_cast<uint8_t>((nscolor >> 24) & 0xFF)
    };
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
    
    // Parse the color - structured components
    auto result = parse_color(color_str);
    
    if (result.success) {
        std::cout << "✓ Successfully parsed color (structured)!" << std::endl;
        print_color_components(result);
    } else {
        std::cout << "✗ Failed to parse color: " << std::string(result.error_message) << std::endl;
    }
    std::cout << std::endl;
    
    // Parse the color as nscolor (uint32 RGBA)
    auto nscolor_result = parse_color_to_nscolor(color_str);
    
    if (nscolor_result.success) {
        auto rgba = nscolor_to_rgba(nscolor_result.nscolor);
        std::cout << "✓ Successfully parsed color (nscolor)!" << std::endl;
        std::cout << "  nscolor: 0x" << std::hex << std::setw(8) << std::setfill('0') 
                  << nscolor_result.nscolor << std::dec << std::endl;
        std::cout << "  RGBA: (" << static_cast<int>(rgba.r) << ", " 
                  << static_cast<int>(rgba.g) << ", " 
                  << static_cast<int>(rgba.b) << ", " 
                  << static_cast<int>(rgba.a) << ")" << std::endl;
    }
    std::cout << std::endl;
    
    // Additional color parsing examples
    std::cout << "=== Additional Color Examples ===" << std::endl;
    std::cout << std::endl;
    
    // Example 1: Named color
    std::cout << "1. Named color (red):" << std::endl;
    auto red_nscolor = parse_color_to_nscolor("red");
    if (red_nscolor.success) {
        auto rgba = nscolor_to_rgba(red_nscolor.nscolor);
        std::cout << "  nscolor: 0x" << std::hex << std::setw(8) << std::setfill('0') 
                  << red_nscolor.nscolor << std::dec << std::endl;
        std::cout << "  RGBA: (" << static_cast<int>(rgba.r) << ", " 
                  << static_cast<int>(rgba.g) << ", " 
                  << static_cast<int>(rgba.b) << ", " 
                  << static_cast<int>(rgba.a) << ")" << std::endl;
    }
    std::cout << std::endl;
    
    // Example 2: Hex color
    std::cout << "2. Hex color (#00ff00):" << std::endl;
    auto hex_nscolor = parse_color_to_nscolor("#00ff00");
    if (hex_nscolor.success) {
        auto rgba = nscolor_to_rgba(hex_nscolor.nscolor);
        std::cout << "  nscolor: 0x" << std::hex << std::setw(8) << std::setfill('0') 
                  << hex_nscolor.nscolor << std::dec << std::endl;
        std::cout << "  RGBA: (" << static_cast<int>(rgba.r) << ", " 
                  << static_cast<int>(rgba.g) << ", " 
                  << static_cast<int>(rgba.b) << ", " 
                  << static_cast<int>(rgba.a) << ")" << std::endl;
    }
    std::cout << std::endl;
    
    // Example 3: RGBA color with alpha
    std::cout << "3. RGBA color (rgba(0, 128, 255, 0.5)):" << std::endl;
    auto rgba_nscolor = parse_color_to_nscolor("rgba(0, 128, 255, 0.5)");
    if (rgba_nscolor.success) {
        auto rgba = nscolor_to_rgba(rgba_nscolor.nscolor);
        std::cout << "  nscolor: 0x" << std::hex << std::setw(8) << std::setfill('0') 
                  << rgba_nscolor.nscolor << std::dec << std::endl;
        std::cout << "  RGBA: (" << static_cast<int>(rgba.r) << ", " 
                  << static_cast<int>(rgba.g) << ", " 
                  << static_cast<int>(rgba.b) << ", " 
                  << static_cast<int>(rgba.a) << ")" << std::endl;
        std::cout << "  Note: Alpha 0.5 = " << static_cast<int>(rgba.a) << "/255" << std::endl;
    }
    std::cout << std::endl;
    
    // Example 4: HSL color (converts to sRGB)
    std::cout << "4. HSL color (hsl(120, 100%, 50%)) - auto-converted to sRGB:" << std::endl;
    auto hsl_nscolor = parse_color_to_nscolor("hsl(120, 100%, 50%)");
    if (hsl_nscolor.success) {
        auto rgba = nscolor_to_rgba(hsl_nscolor.nscolor);
        std::cout << "  nscolor: 0x" << std::hex << std::setw(8) << std::setfill('0') 
                  << hsl_nscolor.nscolor << std::dec << std::endl;
        std::cout << "  RGBA: (" << static_cast<int>(rgba.r) << ", " 
                  << static_cast<int>(rgba.g) << ", " 
                  << static_cast<int>(rgba.b) << ", " 
                  << static_cast<int>(rgba.a) << ")" << std::endl;
    }
    std::cout << std::endl;
    
    std::cout << "=== nscolor Format ===" << std::endl;
    std::cout << "The nscolor uint32 format (little-endian RGBA) is compatible with:" << std::endl;
    std::cout << "  - Mozilla nscolor" << std::endl;
    std::cout << "  - Qt QRgb" << std::endl;
    std::cout << "  - Other RGBA uint32 formats" << std::endl;
    std::cout << std::endl;
    
    std::cout << "Use parse_color_to_nscolor() for easy integration with GUI frameworks!" << std::endl;
    std::cout << std::endl;
    
    std::cout << "=== Example Complete ===" << std::endl;
    
    return 0;
}
