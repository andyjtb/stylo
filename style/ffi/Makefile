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
CARGO_TARGET_DIR = ../../../target
BUILD_DIR = $(CARGO_TARGET_DIR)/debug
STYLO_BUILD_DIR = $(shell find $(BUILD_DIR)/build -type d -name "stylo-*" | head -1)

# Include and library directories
INCLUDES = -I$(STYLO_BUILD_DIR)/out/cxxbridge/include
LDFLAGS = -L$(STYLO_BUILD_DIR)/out -L$(BUILD_DIR)
LIBS = -lstylo_css_parser_ffi -lstylo -lpthread -ldl -lm

# Targets
.PHONY: all example clean rust-lib

all: example

# Build the Rust library first
rust-lib:
	@echo "Building Rust library..."
	cd ../../.. && $(CARGO) build

# Build the C++ example
example: rust-lib example.cpp
	@echo "Building C++ example..."
	@echo "Stylo Build Dir: $(STYLO_BUILD_DIR)"
	$(CXX) $(CXXFLAGS) $(INCLUDES) -o $@ example.cpp $(LDFLAGS) $(LIBS)
	@echo "Built: ./example"
	@echo "Note: You may need to set LD_LIBRARY_PATH=$(STYLO_BUILD_DIR)/out:$(BUILD_DIR) to run the example"

# Run the example
run: example
	LD_LIBRARY_PATH=$(STYLO_BUILD_DIR)/out:$(BUILD_DIR) ./example

# Clean build artifacts
clean:
	rm -f example
	@echo "Cleaned build artifacts"

# Help
help:
	@echo "Available targets:"
	@echo "  all      - Build everything (default)"
	@echo "  example  - Build the C++ example"
	@echo "  run      - Build and run the example"
	@echo "  clean    - Remove build artifacts"
	@echo "  help     - Show this help message"
