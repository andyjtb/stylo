//! Build-time codegen for C++ headers exposing web_atoms::LocalName atom u64 values.
//!
//! Re-implements the atom-packing math from `string_cache_codegen` so we can emit
//! constexpr `uint64_t` values into a C++ header at build time. Drift from the real
//! string_cache packing is caught by `dom_atoms::tests::cpp_atom_packing_matches_string_cache`.

#![allow(dead_code)] // wired up incrementally across the implementation

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

// Tag bits and shifts — must match string_cache::atom internals.
// See string_cache-0.9.0/src/atom.rs:26-34.
const INLINE_TAG: u64 = 0b01;
const STATIC_TAG: u64 = 0b10;
const LEN_OFFSET: u64 = 4;
const STATIC_SHIFT_BITS: u64 = 32;
const MAX_INLINE_LEN: usize = 7;

/// One name from local_names.txt with its computed PascalCase form.
pub struct LocalNameEntry {
    /// The lowercased original name as it appears in local_names.txt.
    pub lower: String,
    /// PascalCase form usable as a C++ identifier.
    pub pascal: String,
}

/// Custom titlecase mappings for special attribute names.
/// Mirrors the mappings previously in html5ever/web_atoms/cpp_codegen.rs.
const CUSTOM_PASCAL_MAPPINGS: &[(&str, &str)] = &[
    ("role", "AccessibilityRole"),
    ("aria-label", "AccessibilityTitle"),
    ("aria-description", "AccessibilityDescription"),
    ("aria-details", "AccessibilityHelpText"),
    ("data-testid", "DataTestId"),
    ("uilayoutstype", "Uilayoutstype"),
    ("onupdate:modelvalue", "ModelValueUpdate"),
];

fn to_pascal_case(s: &str) -> String {
    for &(key, value) in CUSTOM_PASCAL_MAPPINGS {
        if s == key {
            return value.to_string();
        }
    }
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '-' || c == ':' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    if result
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        result = format!("_{}", result);
    }
    result
}

fn is_valid_cpp_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Read local_names.txt, deduplicate by lowercase, drop entries that don't form
/// a valid C++ identifier in PascalCase, and sort by PascalCase form.
pub fn process_local_names(local_names_path: &Path) -> Vec<LocalNameEntry> {
    let file = File::open(local_names_path)
        .unwrap_or_else(|e| panic!("Failed to open {}: {}", local_names_path.display(), e));
    let reader = BufReader::new(file);

    let mut seen = HashSet::new();
    let mut entries: Vec<LocalNameEntry> = Vec::new();

    for line in reader.lines() {
        let name = line.unwrap();
        let lower = name.to_ascii_lowercase();
        if lower.is_empty() {
            continue;
        }
        if seen.insert(lower.clone()) {
            let pascal = to_pascal_case(&lower);
            if !is_valid_cpp_identifier(&pascal) {
                continue;
            }
            entries.push(LocalNameEntry { lower, pascal });
        }
    }

    entries.sort_by(|a, b| a.pascal.cmp(&b.pascal));
    entries
}

/// Pack a string ≤7 bytes into an inline LocalName u64.
///
/// Mirrors the inline packing in `string_cache-0.9.0/src/atom.rs:114-128`:
/// `INLINE_TAG | (len << LEN_OFFSET) | bytes_at(i*8 + 8)`.
fn pack_inline(s: &str) -> u64 {
    debug_assert!(s.len() <= MAX_INLINE_LEN && !s.is_empty());
    let mut value = INLINE_TAG | ((s.len() as u64) << LEN_OFFSET);
    for (i, b) in s.bytes().enumerate() {
        value |= (b as u64) << (i * 8 + 8);
    }
    value
}

/// Compute the packed u64 for every name we'll need to expose to C++.
///
/// Mirrors what `string_cache_codegen::AtomType` does with its input set
/// (see `string_cache_codegen-0.6.1/lib.rs:207-222`):
///
/// 1. Insert every name from local_names.txt PLUS its lowercase variant
///    (matches `web_atoms/build.rs:50-54` which calls `.atom(name)` and
///    `.atom(&name.to_ascii_lowercase())`).
/// 2. Insert the empty string (string_cache_codegen auto-adds it at lib.rs:208).
/// 3. Partition a BTreeSet iterator by `len > 7 || empty` → static set vs inline set.
/// 4. Run `phf_generator::generate_hash` on the static set to get phf indices.
/// 5. Pack each name accordingly.
pub fn compute_atom_packings(local_names_path: &Path) -> BTreeMap<String, u64> {
    let file = File::open(local_names_path)
        .unwrap_or_else(|e| panic!("Failed to open {}: {}", local_names_path.display(), e));
    let reader = BufReader::new(file);

    // Step 1+2: build the same BTreeSet that string_cache_codegen sees.
    let mut all: BTreeSet<String> = BTreeSet::new();
    all.insert(String::new()); // string_cache_codegen auto-adds the empty string
    for line in reader.lines() {
        let name = line.unwrap();
        if name.is_empty() {
            continue;
        }
        all.insert(name.to_ascii_lowercase());
        all.insert(name);
    }

    // Step 3: partition. BTreeSet iteration is already sorted, so static_strs
    // and inline_strs preserve the same lexicographic order string_cache_codegen
    // sees.
    let (static_strs, inline_strs): (Vec<&str>, Vec<&str>) = all
        .iter()
        .map(String::as_str)
        .partition(|s| s.len() > MAX_INLINE_LEN || s.is_empty());

    // Step 4: phf hash for static atoms.
    let phf = phf_generator::generate_hash(&static_strs);

    // Step 5: pack.
    let mut result: BTreeMap<String, u64> = BTreeMap::new();
    for s in &inline_strs {
        result.insert((*s).to_string(), pack_inline(s));
    }
    // phf.map[new_idx] = old_idx; we want each static_strs[old_idx]'s new_idx.
    for (new_idx, &old_idx) in phf.map.iter().enumerate() {
        let s = static_strs[old_idx];
        let packed = STATIC_TAG | ((new_idx as u64) << STATIC_SHIFT_BITS);
        result.insert(s.to_string(), packed);
    }

    result
}

/// Generate `ui_attributes.h` containing the Attribute enum, the constexpr u64
/// array of packed atom values, and the AttributeName struct definition.
pub fn generate_attribute_header(local_names_path: &Path, output_dir: &Path) {
    let entries = process_local_names(local_names_path);
    let packings = compute_atom_packings(local_names_path);

    let header_path = output_dir.join("ui_attributes.h");
    let mut out = BufWriter::new(File::create(&header_path).unwrap_or_else(|e| {
        panic!("Failed to create {}: {}", header_path.display(), e)
    }));

    writeln!(out, "// Auto-generated by stylo build_cpp_codegen.rs - DO NOT EDIT").unwrap();
    writeln!(out, "#pragma once").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "#include <cstddef>").unwrap();
    writeln!(out, "#include <cstdint>").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "namespace ui::dom {{").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "/// Enum representing HTML/SVG attribute and element names.").unwrap();
    writeln!(out, "/// Backed by web_atoms::LocalName atoms (string_cache).").unwrap();
    writeln!(out, "enum class Attribute : uint16_t").unwrap();
    writeln!(out, "{{").unwrap();
    for (i, e) in entries.iter().enumerate() {
        writeln!(out, "    {} = {},", e.pascal, i).unwrap();
    }
    writeln!(out, "}};").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "constexpr size_t attributeCount = {};", entries.len()).unwrap();
    writeln!(out).unwrap();

    writeln!(out, "/// Packed LocalName u64 values, indexed by Attribute enum.").unwrap();
    writeln!(out, "/// Each value is the bit-identical u64 that string_cache produces").unwrap();
    writeln!(out, "/// for the corresponding atom — STATIC_TAG | (phf_idx << 32) for").unwrap();
    writeln!(out, "/// long names, INLINE_TAG | (len << 4) | bytes for short ones.").unwrap();
    writeln!(out, "inline constexpr uint64_t ATTRIBUTE_RAW_ATOMS[attributeCount] = {{").unwrap();
    for e in &entries {
        let packed = packings
            .get(&e.lower)
            .unwrap_or_else(|| panic!("no packed value for {:?}", e.lower));
        writeln!(out, "    0x{:016x}ULL, // {}", packed, e.lower).unwrap();
    }
    writeln!(out, "}};").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "/// Constexpr-constructible name handle for a known attribute.").unwrap();
    writeln!(out, "/// Implicit conversion from `Attribute` enum is intentional.").unwrap();
    writeln!(out, "struct AttributeName {{").unwrap();
    writeln!(out, "    uint64_t repr;").unwrap();
    writeln!(out, "    constexpr /*implicit*/ AttributeName(Attribute id) noexcept").unwrap();
    writeln!(out, "        : repr(ATTRIBUTE_RAW_ATOMS[static_cast<size_t>(id)]) {{}}").unwrap();
    writeln!(out, "    constexpr bool operator==(AttributeName const&) const = default;").unwrap();
    writeln!(out, "}};").unwrap();
    writeln!(out).unwrap();

    writeln!(out, "}} // namespace ui::dom").unwrap();
}

/// Read `longhandCount` out of the just-generated `ui_css_properties.h`.
///
/// `build.py` writes `ui_css_properties.h` into the same output directory
/// we're about to emit into, and the file contains a single
/// `constexpr size_t longhandCount = N;` line. We parse N directly so the
/// ChangedProperties storage array size stays in lockstep with
/// `property_counts::LONGHANDS` without duplicating engine filtering.
pub fn read_longhand_count(css_props_header: &Path) -> usize {
    let body = std::fs::read_to_string(css_props_header).unwrap_or_else(|e| {
        panic!(
            "Failed to read {}: {}. \
             generate_changed_properties_header must run after build.py writes ui_css_properties.h.",
            css_props_header.display(),
            e
        )
    });
    for line in body.lines() {
        let trimmed = line.trim();
        // Expecting exactly: "constexpr size_t longhandCount = N;"
        if let Some(rest) = trimmed.strip_prefix("constexpr size_t longhandCount") {
            let rest = rest.trim_start_matches(|c: char| c == ' ' || c == '=');
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<usize>() {
                return n;
            }
        }
    }
    panic!(
        "Could not find `constexpr size_t longhandCount = N;` in {}",
        css_props_header.display()
    )
}

/// Generate `ui_changed_properties.h` containing a layout-stable
/// `ChangedProperties` POD that mirrors Rust's `LonghandIdSet`.
pub fn generate_changed_properties_header(longhand_count: usize, output_dir: &Path) {
    // Matches style/properties/mod.rs:1225:
    //   storage: [u32; ((property_counts::LONGHANDS as usize) - 1 + 32) / 32],
    let storage_words = ((longhand_count - 1) + 32) / 32;
    let header_path = output_dir.join("ui_changed_properties.h");
    let mut out = BufWriter::new(File::create(&header_path).unwrap_or_else(|e| {
        panic!("Failed to create {}: {}", header_path.display(), e)
    }));

    writeln!(out, "// Auto-generated by stylo build_cpp_codegen.rs - DO NOT EDIT").unwrap();
    writeln!(out, "#pragma once").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "#include <cstddef>").unwrap();
    writeln!(out, "#include <cstdint>").unwrap();
    writeln!(out, "#include \"ui_css_properties.h\"").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "namespace ui::css {{").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "/// Layout-equivalent to Rust's LonghandIdSet (style/properties/mod.rs).").unwrap();
    writeln!(out, "/// Bit indices match the longhand portion of PropertyId 1:1.").unwrap();
    writeln!(out, "struct ChangedProperties {{").unwrap();
    writeln!(out, "    uint32_t storage[{}];", storage_words).unwrap();
    writeln!(out).unwrap();
    writeln!(out, "    constexpr bool contains(PropertyId id) const noexcept {{").unwrap();
    writeln!(out, "        const auto bit = static_cast<size_t>(id);").unwrap();
    writeln!(out, "        return (storage[bit / 32] & (1u << (bit % 32))) != 0;").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "    class iterator;").unwrap();
    writeln!(out, "    iterator begin() const noexcept;").unwrap();
    writeln!(out, "    iterator end() const noexcept;").unwrap();
    writeln!(out, "}};").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "static_assert(sizeof(ChangedProperties) == {} * sizeof(uint32_t),", storage_words).unwrap();
    writeln!(out, "    \"ChangedProperties must be POD with no padding\");").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "}} // namespace ui::css").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_inline_id() {
        // "id" = bytes 0x69, 0x64; len = 2.
        // Expected: INLINE_TAG (0b01) | (2 << 4) | (0x69 << 8) | (0x64 << 16)
        //         = 0x01 | 0x20 | 0x6900 | 0x640000
        //         = 0x646921
        assert_eq!(pack_inline("id"), 0x00646921);
    }

    #[test]
    fn pack_inline_a() {
        // "a" = byte 0x61; len = 1.
        // Expected: 0x01 | (1 << 4) | (0x61 << 8) = 0x6111
        assert_eq!(pack_inline("a"), 0x6111);
    }

    #[test]
    fn pack_inline_div() {
        // "div" = 0x64,0x69,0x76; len = 3.
        // Expected: 0x01 | (3 << 4) | 0x64<<8 | 0x69<<16 | 0x76<<24
        //         = 0x31 | 0x6400 | 0x690000 | 0x76000000
        //         = 0x76696431
        assert_eq!(pack_inline("div"), 0x76696431);
    }

    #[test]
    fn pack_static_round_trips_for_long_names() {
        // "contenteditable" is 15 bytes → must be a static atom.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("html5ever/web_atoms/local_names.txt");
        let packings = compute_atom_packings(&path);
        let v = packings
            .get("contenteditable")
            .expect("contenteditable in static set");
        // Tag bits must be STATIC_TAG.
        assert_eq!(
            v & 0b11,
            STATIC_TAG,
            "expected STATIC tag for contenteditable, got {:#x}",
            v
        );
    }

    #[test]
    fn read_longhand_count_parses_generated_header() {
        let tmp = std::env::temp_dir().join("stylo_longhand_count_test.h");
        std::fs::write(
            &tmp,
            "#pragma once\n\
             namespace ui::css {\n\
             enum class PropertyId : uint16_t { Foo = 0 };\n\
             constexpr size_t longhandCount = 245;\n\
             constexpr size_t shorthandCount = 42;\n\
             } // namespace ui::css\n",
        )
        .unwrap();
        assert_eq!(read_longhand_count(&tmp), 245);
    }

    #[test]
    fn generate_attribute_header_writes_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("html5ever/web_atoms/local_names.txt");
        let tmp = std::env::temp_dir().join("stylo_cpp_codegen_test");
        std::fs::create_dir_all(&tmp).unwrap();
        generate_attribute_header(&path, &tmp);
        let body = std::fs::read_to_string(tmp.join("ui_attributes.h")).unwrap();
        assert!(body.contains("enum class Attribute : uint16_t"));
        assert!(body.contains("ATTRIBUTE_RAW_ATOMS"));
        assert!(body.contains("struct AttributeName"));
        assert!(body.contains("// id"), "expected the 'id' atom comment");
    }

    #[test]
    fn process_local_names_returns_sorted_entries() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("html5ever/web_atoms/local_names.txt");
        let entries = process_local_names(&path);
        assert!(entries.len() > 100, "expected many entries, got {}", entries.len());
        // PascalCase ordering: every entry's pascal should be lex-≤ next.
        for w in entries.windows(2) {
            assert!(w[0].pascal <= w[1].pascal, "{} > {}", w[0].pascal, w[1].pascal);
        }
        // "id" should be present.
        assert!(entries.iter().any(|e| e.lower == "id"));
    }
}
