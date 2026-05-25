//! Tests for complex lvalue expressions: struct field access, array/slice
//! indexing, tuple fields, and dereferenced raw pointers via Box.
//!
//! The macro strips the `++`/`--` tokens and parses the remainder as a
//! `syn::Expr`, so any valid mutable place expression should work.

use crement::{crement, post_dec, post_inc, pre_dec, pre_inc};

// ── Struct field access ───────────────────────────────────────────────────────

struct Scoreboard {
    hits: i32,
    misses: i32,
}

#[test]
fn field_prefix_inc() {
    let mut sb = Scoreboard { hits: 0, misses: 0 };
    assert_eq!(crement!(++sb.hits), 1);
    assert_eq!(sb.hits, 1);
    assert_eq!(sb.misses, 0); // other field untouched
}

#[test]
fn field_postfix_inc_returns_old() {
    let mut sb = Scoreboard { hits: 5, misses: 2 };
    let old_hits = crement!(sb.hits++);
    assert_eq!(old_hits, 5);
    assert_eq!(sb.hits, 6);
}

#[test]
fn field_prefix_dec() {
    let mut sb = Scoreboard { hits: 10, misses: 3 };
    assert_eq!(pre_dec!(sb.misses), 2);
    assert_eq!(sb.misses, 2);
    assert_eq!(sb.hits, 10);
}

#[test]
fn field_postfix_dec_returns_old() {
    let mut sb = Scoreboard { hits: 10, misses: 5 };
    let old = post_dec!(sb.misses);
    assert_eq!(old, 5);
    assert_eq!(sb.misses, 4);
}

#[test]
fn two_fields_independent() {
    let mut sb = Scoreboard { hits: 0, misses: 0 };
    crement!(++sb.hits);
    crement!(++sb.hits);
    crement!(++sb.misses);
    assert_eq!(sb.hits, 2);
    assert_eq!(sb.misses, 1);
}

// ── Nested struct field ───────────────────────────────────────────────────────

struct Outer {
    inner: Inner,
}

struct Inner {
    value: i32,
}

#[test]
fn nested_field_postfix_inc() {
    let mut o = Outer { inner: Inner { value: 7 } };
    let old = crement!(o.inner.value++);
    assert_eq!(old, 7);
    assert_eq!(o.inner.value, 8);
}

// ── Array / slice indexing ────────────────────────────────────────────────────

#[test]
fn array_index_prefix_inc() {
    let mut arr = [10i32, 20, 30, 40];
    assert_eq!(crement!(++arr[2]), 31);
    assert_eq!(arr, [10, 20, 31, 40]);
}

#[test]
fn array_index_postfix_inc_returns_old() {
    let mut arr = [10i32, 20, 30, 40];
    let old = crement!(arr[1]++);
    assert_eq!(old, 20);
    assert_eq!(arr, [10, 21, 30, 40]);
}

#[test]
fn array_index_prefix_dec() {
    let mut arr = [0i32, 0, 5, 0];
    assert_eq!(pre_dec!(arr[2]), 4);
    assert_eq!(arr[2], 4);
}

#[test]
fn array_index_postfix_dec_returns_old() {
    let mut arr = [100u64; 4];
    let old = post_dec!(arr[3]);
    assert_eq!(old, 100u64);
    assert_eq!(arr[3], 99);
}

#[test]
fn slice_index_postfix_inc() {
    let mut data = vec![1i32, 2, 3, 4, 5];
    let slice: &mut [i32] = &mut data;
    let old = post_inc!(slice[0]);
    assert_eq!(old, 1);
    assert_eq!(slice[0], 2);
}

#[test]
fn array_walk_with_postfix() {
    // Use a for-loop with an explicit `val` so that the RHS of `arr[..] = val`
    // is a plain variable rather than something that also reads `w`, avoiding
    // any confusion with Rust's right-to-left evaluation of assignment operands.
    let mut arr = [0i32; 5];
    let mut w: usize = 0;
    for val in 0..5i32 {
        arr[post_inc!(w)] = val;
    }
    assert_eq!(arr, [0, 1, 2, 3, 4]);
    assert_eq!(w, 5);
}

// ── Tuple field indexing ──────────────────────────────────────────────────────

#[test]
fn tuple_field_prefix_inc() {
    let mut t: (i32, i32) = (10, 20);
    assert_eq!(pre_inc!(t.0), 11);
    assert_eq!(t.0, 11);
    assert_eq!(t.1, 20);
}

#[test]
fn tuple_field_postfix_dec() {
    let mut t: (i32, i32) = (5, 3);
    let old = post_dec!(t.1);
    assert_eq!(old, 3);
    assert_eq!(t.1, 2);
}

// ── Dereference (Box<T> as a stand-in for raw pointers in safe code) ──────────

#[test]
fn boxed_value_prefix_inc() {
    let mut b: Box<i32> = Box::new(41);
    assert_eq!(pre_inc!(*b), 42);
    assert_eq!(*b, 42);
}

#[test]
fn boxed_value_postfix_inc_returns_old() {
    let mut b: Box<i32> = Box::new(99);
    let old = post_inc!(*b);
    assert_eq!(old, 99);
    assert_eq!(*b, 100);
}

#[test]
fn boxed_value_postfix_dec() {
    let mut b: Box<i64> = Box::new(10);
    let old = crement!(*b--);
    assert_eq!(old, 10i64);
    assert_eq!(*b, 9);
}

// ── Vec element ───────────────────────────────────────────────────────────────

#[test]
fn vec_element_postfix_inc() {
    let mut v: Vec<i32> = vec![1, 2, 3];
    let old = post_inc!(v[1]);
    assert_eq!(old, 2);
    assert_eq!(v[1], 3);
}

#[test]
fn vec_element_prefix_dec() {
    let mut v: Vec<u32> = vec![10, 20, 30];
    assert_eq!(pre_dec!(v[2]), 29u32);
    assert_eq!(v[2], 29);
}
