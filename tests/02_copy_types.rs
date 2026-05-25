//! Tests across the full range of numeric Copy types that Rust ships:
//! signed/unsigned integers of every width, and floating-point types.
//! For Copy types the Clone call is optimised away by the compiler — these
//! tests confirm correctness of the generated code regardless.

use crement::{crement, post_dec, post_inc, pre_dec, pre_inc};

// ── Signed integers ───────────────────────────────────────────────────────────

#[test]
fn i8_prefix_inc() {
    let mut x: i8 = 10;
    assert_eq!(crement!(++x), 11i8);
    assert_eq!(x, 11);
}

#[test]
fn i8_postfix_inc() {
    let mut x: i8 = 10;
    assert_eq!(crement!(x++), 10i8);
    assert_eq!(x, 11);
}

#[test]
fn i16_roundtrip() {
    let mut x: i16 = 1000;
    assert_eq!(post_inc!(x), 1000i16);
    assert_eq!(pre_inc!(x), 1002i16);
    assert_eq!(post_dec!(x), 1002i16);
    assert_eq!(pre_dec!(x), 1000i16);
    assert_eq!(x, 1000);
}

#[test]
fn i32_full_roundtrip() {
    let mut x: i32 = 0;
    assert_eq!(pre_inc!(x), 1);
    assert_eq!(post_inc!(x), 1);
    assert_eq!(x, 2);
    assert_eq!(pre_dec!(x), 1);
    assert_eq!(post_dec!(x), 1);
    assert_eq!(x, 0);
}

#[test]
fn i64_postfix_sequence() {
    let mut x: i64 = 100;
    let a = post_inc!(x);
    let b = post_inc!(x);
    let c = post_inc!(x);
    assert_eq!([a, b, c], [100i64, 101, 102]);
    assert_eq!(x, 103);
}

#[test]
fn i128_prefix_dec() {
    let mut x: i128 = 0;
    assert_eq!(pre_dec!(x), -1i128);
    assert_eq!(x, -1);
}

#[test]
fn isize_postfix_dec() {
    let mut x: isize = 5;
    assert_eq!(post_dec!(x), 5isize);
    assert_eq!(x, 4);
}

// ── Unsigned integers ─────────────────────────────────────────────────────────

#[test]
fn u8_prefix_inc() {
    let mut x: u8 = 200;
    assert_eq!(pre_inc!(x), 201u8);
    assert_eq!(x, 201);
}

#[test]
fn u16_postfix_inc() {
    let mut x: u16 = 0;
    assert_eq!(post_inc!(x), 0u16);
    assert_eq!(x, 1);
}

#[test]
fn u32_full_roundtrip() {
    let mut x: u32 = 50;
    assert_eq!(pre_inc!(x), 51u32);
    assert_eq!(post_inc!(x), 51u32);
    assert_eq!(x, 52);
    assert_eq!(pre_dec!(x), 51u32);
    assert_eq!(post_dec!(x), 51u32);
    assert_eq!(x, 50);
}

#[test]
fn u64_postfix_sequence() {
    let mut x: u64 = 1_000_000;
    let a = post_inc!(x);
    let b = post_inc!(x);
    assert_eq!(a, 1_000_000u64);
    assert_eq!(b, 1_000_001u64);
    assert_eq!(x, 1_000_002);
}

#[test]
fn u128_prefix_inc() {
    let mut x: u128 = u64::MAX as u128;
    let ret = pre_inc!(x);
    assert_eq!(ret, u64::MAX as u128 + 1);
}

#[test]
fn usize_postfix_dec() {
    let mut x: usize = 10;
    assert_eq!(post_dec!(x), 10usize);
    assert_eq!(x, 9);
}

// ── Floating-point note ───────────────────────────────────────────────────────
//
// `f32` and `f64` implement `AddAssign<f32>` / `SubAssign<f64>` (same-type
// RHS), but NOT `AddAssign<{integer}>`.  Because the macro generates `+= 1`
// with an unsuffixed integer literal, float types are deliberately out of
// scope for `crement!` — use `x += 1.0` directly for floats.  This matches
// the spirit of C's `++`/`--` which is primarily for integral types.

// ── bool is Copy but doesn't implement AddAssign — this is a compile-time
//    restriction enforced by the trait system, not something we test here.

// ── char is Copy; char += 1 is not valid Rust, so it's also excluded by the
//    trait system without any special-casing in the macro.
