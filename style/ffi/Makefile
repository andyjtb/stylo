# Makefile for building C++ examples using Stylo CSS Parser FFI
#
# Usage:
#   make example    - Build the example program
#   make clean      - Clean build artifacts
#
# Prerequisites:
#   - Rust toolchain (cargo)
#   - C++ compiler (g++ or clang++)
#   - Make

# Configuration
CXX ?= g++
CXXFLAGS = -std=c++14 -Wall -Wextra
CARGO = cargo

# Build directories
CARGO_TARGET_DIR = ../../target
BUILD_DIR = $(CARGO_TARGET_DIR)/debug
STYLO_BUILD_DIR = $(shell find $(BUILD_DIR)/build -type d -name "stylo-*" | head -1)

# Include and library directories
INCLUDES = -I$(STYLO_BUILD_DIR)/out/cxxbridge/include
CXX_LIB_DIR = $(shell find $(BUILD_DIR)/build -type f -name "libcxxbridge1.a" -exec dirname {} \; | head -1)
LDFLAGS = -L$(STYLO_BUILD_DIR)/out -L$(CXX_LIB_DIR) -L$(BUILD_DIR)
LIBS = -Wl,--whole-archive -lstyle -Wl,--no-whole-archive -lcxxbridge1 -lpthread -ldl -lm -lgcc_s

# Targets
.PHONY: all example color_parser clean rust-lib

all: example color_parser

# Build the Rust library first
rust-lib:
	@echo "Building Rust library..."
	cd ../.. && $(CARGO) build

# Build the C++ example
example: rust-lib example.cpp
	@echo "Building C++ example..."
	@echo "Stylo Build Dir: $(STYLO_BUILD_DIR)"
	$(CXX) $(CXXFLAGS) $(INCLUDES) -o $@ example.cpp $(LDFLAGS) $(LIBS)
	@echo "Built: ./example"
	@echo "Note: You may need to set LD_LIBRARY_PATH=$(STYLO_BUILD_DIR)/out:$(BUILD_DIR) to run the example"

# Build the color_parser example
color_parser: rust-lib color_parser.cpp
	@echo "Building C++ color_parser example..."
	@echo "Stylo Build Dir: $(STYLO_BUILD_DIR)"
	$(CXX) $(CXXFLAGS) $(INCLUDES) -o $@ color_parser.cpp $(LDFLAGS) $(LIBS)
	@echo "Built: ./color_parser"
	@echo "Note: You may need to set LD_LIBRARY_PATH=$(STYLO_BUILD_DIR)/out:$(BUILD_DIR) to run the color_parser"

# Run the example
run: example
	LD_LIBRARY_PATH=$(STYLO_BUILD_DIR)/out:$(BUILD_DIR) ./example

# Run the color_parser example
run_color_parser: color_parser
	LD_LIBRARY_PATH=$(STYLO_BUILD_DIR)/out:$(BUILD_DIR) ./color_parser

# Clean build artifacts
clean:
	rm -f example color_parser
	@echo "Cleaned build artifacts"

# Help
help:
	@echo "Available targets:"
	@echo "  all           - Build everything (default)"
	@echo "  example       - Build the C++ example"
	@echo "  color_parser  - Build the C++ color_parser example"
	@echo "  run           - Build and run the example"
	@echo "  run_color_parser - Build and run the color_parser example"
	@echo "  clean         - Remove build artifacts"
	@echo "  help          - Show this help message"
