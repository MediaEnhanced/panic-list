//! plel Rust Library
//!
//! Basic Operation functions that Could or Won't Panic
#![no_std]
#[cfg(feature = "std")]
extern crate std;
// Implicitly Defined
#[cfg(not(feature = "std"))]
extern crate core;

/// Add without causing a panic!\
/// Result is expected to wrap on overflow by design.
pub fn add_no_panic(a: usize, b: usize) -> usize {
    a.wrapping_add(b)
}

/// Subtract without causing a panic!\
/// Result is expected to saturated to zero instead of overflow by design.
pub fn sub_no_panic(a: usize, b: usize) -> usize {
    a.saturating_sub(b)
}

/// Multiply without causing a panic!\
/// Result is checked and function will return None if it overflowed.
pub fn mult_no_panic(a: usize, b: usize) -> Option<usize> {
    a.checked_mul(b)
}

/// Divide with integer truncation without causing a panic!\
/// Result will be valid because the dividend can never be zero.
pub fn div_no_panic(a: usize, b: core::num::NonZeroUsize) -> usize {
    a / b
}

/// Get Data Slice Byte without causing a panic!\
/// Index is checked and function will return None if index was out of bounds.
pub fn slice_byte_no_panic(data: &[u8], index: usize) -> Option<u8> {
    Some(*(data.get(index)?))
}

/// Internal First Entry Function shouldn't panic since the compiler
/// is told that the slice is never considered empty.
#[inline(never)]
fn first_entry_internal_no_panic(data: &[usize]) -> usize {
    #[allow(unsafe_code)]
    unsafe {
        core::hint::assert_unchecked(!data.is_empty())
    }
    data[0]
}

/// Add Entries without causing a panic due to accepting the unchecked risk
/// that COULD cause overflow and might lead to program bugs.
pub fn add_entries_no_panic(a: usize, data: &[usize]) -> usize {
    #[allow(unsafe_code)]
    unsafe {
        a.unchecked_add(first_entry_internal_no_panic(data))
    }
}

#[cfg(feature = "panic-possible")]
/// Possible To Panic Functions
pub mod possible {
    /// Add Raw
    pub fn add(a: usize, b: usize) -> usize {
        a + b
    }

    /// Subtract Raw
    pub fn sub(a: usize, b: usize) -> usize {
        a - b
    }

    /// Multiply Raw
    pub fn mult(a: usize, b: usize) -> usize {
        a * b
    }

    /// Divide Raw with integer truncation
    pub fn div(a: usize, b: usize) -> usize {
        a / b
    }

    /// Get Slice Byte
    // #[allow(unsafe_code)]
    // #[unsafe(no_mangle)]
    pub fn slice_byte(data: &[u8], index: usize) -> u8 {
        data[index]
    }

    /// Internal First Entry Function
    #[inline(never)]
    fn first_entry_internal(data: &[usize]) -> usize {
        data[0]
    }

    /// Add Entries
    pub fn add_entries(a: usize, data: &[usize]) -> usize {
        a + first_entry_internal(data)
    }
}

#[cfg(feature = "std")]
/// Print Hello World
pub fn print_hello_world() {
    std::println!("Hello World!");
}
