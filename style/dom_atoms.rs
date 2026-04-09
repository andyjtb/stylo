//! DOM atom bridge — runtime FFI for dynamic interning, plus a
//! drift-protection test that the build-time atom packing matches
//! string_cache's runtime representation.

use std::num::NonZeroU64;
use web_atoms::LocalName;

/// FFI-stable representation of a `web_atoms::LocalName`. The Rust
/// `LocalName` is a `string_cache::Atom<...>` whose layout is one
/// `NonZeroU64` (verified by `static_assertions::assert_eq_size!`).
#[repr(transparent)]
pub struct StyloLocalName(pub NonZeroU64);

static_assertions::assert_eq_size!(LocalName, StyloLocalName);
static_assertions::assert_eq_size!(LocalName, u64);
static_assertions::assert_eq_align!(LocalName, u64);

const TAG_MASK: u64 = 0b11;
const DYNAMIC_TAG: u64 = 0b00;

#[inline]
fn is_dynamic(repr: u64) -> bool {
    (repr & TAG_MASK) == DYNAMIC_TAG
}

/// Intern a UTF-8 string as a LocalName atom and return its raw u64.
///
/// # Safety
/// `ptr` must point to `len` valid bytes of UTF-8. Caller must eventually
/// release the result with `stylo_local_name_drop` if the atom is dynamic
/// (tag bits == 0). For static and inline atoms drop is a no-op.
#[no_mangle]
pub unsafe extern "C" fn stylo_local_name_intern(
    ptr: *const u8,
    len: usize,
) -> StyloLocalName {
    let bytes = std::slice::from_raw_parts(ptr, len);
    let s = std::str::from_utf8_unchecked(bytes);
    let atom = LocalName::from(s);
    let u: u64 = std::mem::transmute(atom);
    // SAFETY: LocalName is always non-zero (NonZeroU64 inside).
    StyloLocalName(NonZeroU64::new_unchecked(u))
}

/// Increase the refcount on a LocalName atom.
///
/// For static and inline atoms (tag bits != 0b00) this is a no-op.
/// For dynamic atoms it goes through string_cache's atomic refcount.
#[no_mangle]
pub unsafe extern "C" fn stylo_local_name_clone(a: StyloLocalName) -> StyloLocalName {
    let raw = a.0.get();
    if !is_dynamic(raw) {
        return a; // bitwise copy is fine for static/inline
    }
    // Reconstruct, clone, forget the original to keep the refcount balanced.
    let original: LocalName = std::mem::transmute(raw);
    let cloned = original.clone();
    std::mem::forget(original);
    let cloned_u: u64 = std::mem::transmute(cloned);
    StyloLocalName(NonZeroU64::new_unchecked(cloned_u))
}

/// Decrease the refcount on a LocalName atom.
///
/// For static and inline atoms this is a no-op. For dynamic atoms
/// it drops the LocalName, which decrements the atomic refcount.
#[no_mangle]
pub unsafe extern "C" fn stylo_local_name_drop(a: StyloLocalName) {
    let raw = a.0.get();
    if !is_dynamic(raw) {
        return;
    }
    let owned: LocalName = std::mem::transmute(raw);
    drop(owned);
}

/// Borrow a LocalName atom's UTF-8 bytes. The returned pointer is valid
/// for as long as the caller holds a reference to the atom.
#[no_mangle]
pub unsafe extern "C" fn stylo_local_name_as_str(
    a: StyloLocalName,
    out_len: *mut usize,
) -> *const u8 {
    let raw = a.0.get();
    let atom: LocalName = std::mem::transmute(raw);
    let s: &str = atom.as_ref();
    *out_len = s.len();
    let ptr = s.as_ptr();
    std::mem::forget(atom); // caller still owns the original
    ptr
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_cpp_codegen;
    use std::path::Path;
    use web_atoms::LocalName;

    #[test]
    fn intern_id_matches_inline_packing() {
        let s = "id";
        let result = unsafe { stylo_local_name_intern(s.as_ptr(), s.len()) };
        // From Task 2's pack_inline test: "id" packs to 0x646921.
        assert_eq!(result.0.get(), 0x00646921);
    }

    #[test]
    fn intern_dynamic_string_has_dynamic_tag() {
        let s = "this-is-a-runtime-name-not-in-local-names-txt";
        let result = unsafe { stylo_local_name_intern(s.as_ptr(), s.len()) };
        // Tag bits 0b00 = DYNAMIC_TAG.
        assert_eq!(result.0.get() & 0b11, 0b00);
        // Drop it to release the refcount.
        unsafe { stylo_local_name_drop(result) };
    }

    #[test]
    fn dynamic_atom_clone_drop_round_trip() {
        let s = "another-runtime-name-12345";
        let original = unsafe { stylo_local_name_intern(s.as_ptr(), s.len()) };
        assert!(is_dynamic(original.0.get()));

        // Clone twice, drop three times — refcount should hit zero exactly once.
        let c1 = unsafe { stylo_local_name_clone(StyloLocalName(original.0)) };
        let c2 = unsafe { stylo_local_name_clone(StyloLocalName(original.0)) };
        unsafe {
            stylo_local_name_drop(c1);
            stylo_local_name_drop(c2);
            stylo_local_name_drop(original);
        }
    }

    #[test]
    fn as_str_round_trip_inline() {
        let s = "id";
        let a = unsafe { stylo_local_name_intern(s.as_ptr(), s.len()) };
        let mut len: usize = 0;
        let ptr = unsafe { stylo_local_name_as_str(StyloLocalName(a.0), &mut len) };
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        assert_eq!(bytes, b"id");
        unsafe { stylo_local_name_drop(a) };
    }

    #[test]
    fn as_str_round_trip_dynamic() {
        let s = "yet-another-runtime-name";
        let a = unsafe { stylo_local_name_intern(s.as_ptr(), s.len()) };
        let mut len: usize = 0;
        let ptr = unsafe { stylo_local_name_as_str(StyloLocalName(a.0), &mut len) };
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        assert_eq!(bytes, s.as_bytes());
        unsafe { stylo_local_name_drop(a) };
    }

    #[test]
    fn static_atom_clone_drop_is_no_op() {
        let s = "id";
        let a = unsafe { stylo_local_name_intern(s.as_ptr(), s.len()) };
        let cloned = unsafe { stylo_local_name_clone(StyloLocalName(a.0)) };
        assert_eq!(cloned.0.get(), a.0.get());
        // Both drops are no-ops; calling more than once must not panic.
        unsafe {
            stylo_local_name_drop(cloned);
            stylo_local_name_drop(a);
        }
    }

    /// Verify that our build-time `pack_inline`/`compute_atom_packings` produces
    /// u64 values bit-identical to what `LocalName::from(name)` produces at
    /// runtime via string_cache. If string_cache ever changes its packing
    /// layout, this test fails loudly instead of silently emitting wrong
    /// constexpr values into the C++ header.
    #[test]
    fn cpp_atom_packing_matches_string_cache() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("html5ever/web_atoms/local_names.txt");
        let packings = build_cpp_codegen::compute_atom_packings(&path);

        for (name, &expected_u64) in &packings {
            if name.is_empty() {
                continue;
            }
            let actual = LocalName::from(name.as_str());
            // LocalName is `{ NonZeroU64, PhantomData }` — layout-equal to u64.
            let actual_u64: u64 = unsafe {
                std::mem::transmute_copy::<LocalName, u64>(&actual)
            };
            assert_eq!(
                actual_u64, expected_u64,
                "atom packing drift for {:?}: build-time {:#x} != runtime {:#x}",
                name, expected_u64, actual_u64
            );
        }
    }
}
