//! Tests for Clone-but-NOT-Copy types.
//!
//! This is the interesting case: the macro must capture the old value using
//! `Clone::clone` rather than a bit-copy.  We verify both correctness (the
//! returned old value is independent of subsequent mutations) and that the
//! clone is a deep copy (mutating the original after the call does not alias
//! the captured snapshot).

use std::ops::{AddAssign, SubAssign};

use crement::{crement, post_dec, post_inc, pre_dec, pre_inc};

// ── Helper: a Clone-only counter type ────────────────────────────────────────

/// A newtype that wraps `i64` and deliberately opts out of `Copy`.
/// It records how many times `clone()` was called so tests can assert that
/// the macro calls clone exactly once per postfix operation.
#[derive(Debug, PartialEq, Eq)]
struct Nc {
    val: i64,
    /// Incremented in `clone()`.  Wrapped in a `Box` so that the type is
    /// definitely not `Copy` (a `Box` field blocks the compiler from ever
    /// auto-deriving `Copy`).
    clones: Box<u32>,
}

impl Nc {
    fn new(val: i64) -> Self {
        Self { val, clones: Box::new(0) }
    }
    fn clone_count(&self) -> u32 {
        *self.clones
    }
}

impl Clone for Nc {
    fn clone(&self) -> Self {
        Self {
            val: self.val,
            clones: Box::new(*self.clones + 1),
        }
    }
}

impl AddAssign<i64> for Nc {
    fn add_assign(&mut self, rhs: i64) {
        self.val += rhs;
    }
}

impl SubAssign<i64> for Nc {
    fn sub_assign(&mut self, rhs: i64) {
        self.val -= rhs;
    }
}

// ── Prefix operations (no clone of the result in test assertions needed) ──────

#[test]
fn prefix_inc_clone_only_returns_new_value() {
    let mut x = Nc::new(5);
    let ret = pre_inc!(x);
    assert_eq!(ret.val, 6, "prefix ++ must return new value");
    assert_eq!(x.val, 6, "variable must be mutated");
}

#[test]
fn prefix_dec_clone_only_returns_new_value() {
    let mut x = Nc::new(5);
    let ret = pre_dec!(x);
    assert_eq!(ret.val, 4);
    assert_eq!(x.val, 4);
}

#[test]
fn crement_prefix_inc_clone_only() {
    let mut x = Nc::new(10);
    let ret = crement!(++x);
    assert_eq!(ret.val, 11);
    assert_eq!(x.val, 11);
}

#[test]
fn crement_prefix_dec_clone_only() {
    let mut x = Nc::new(10);
    let ret = crement!(--x);
    assert_eq!(ret.val, 9);
    assert_eq!(x.val, 9);
}

// ── Postfix operations — must clone exactly once and capture the old value ───

#[test]
fn postfix_inc_returns_old_value() {
    let mut x = Nc::new(5);
    let old = post_inc!(x);
    assert_eq!(old.val, 5, "postfix ++ must return the old value");
    assert_eq!(x.val, 6,   "postfix ++ must mutate in place");
}

#[test]
fn postfix_dec_returns_old_value() {
    let mut x = Nc::new(5);
    let old = post_dec!(x);
    assert_eq!(old.val, 5, "postfix -- must return the old value");
    assert_eq!(x.val, 4,   "postfix -- must mutate in place");
}

#[test]
fn crement_postfix_inc_returns_old_value() {
    let mut x = Nc::new(42);
    let old = crement!(x++);
    assert_eq!(old.val, 42);
    assert_eq!(x.val, 43);
}

#[test]
fn crement_postfix_dec_returns_old_value() {
    let mut x = Nc::new(42);
    let old = crement!(x--);
    assert_eq!(old.val, 42);
    assert_eq!(x.val, 41);
}

#[test]
fn postfix_clone_is_independent_of_original() {
    let mut x = Nc::new(7);
    let snap = post_inc!(x);
    // Further mutate x — snap must NOT be affected.
    x.val = 999;
    assert_eq!(snap.val, 7, "snapshot is a deep copy; mutating original must not alias it");
}

#[test]
fn postfix_clone_called_exactly_once() {
    let mut x = Nc::new(0);
    let snap = post_inc!(x);
    // The snapshot was created by exactly one Clone::clone call.
    assert_eq!(snap.clone_count(), 1, "postfix must call clone exactly once");
    // The variable itself was never cloned (it was mutated in place).
    assert_eq!(x.clone_count(), 0, "original must not have been cloned");
}

#[test]
fn prefix_does_not_call_clone_unnecessarily() {
    // Prefix ++ must call clone once (to return the new value) but must NOT
    // call it a second time.
    let mut x = Nc::new(0);
    let ret = pre_inc!(x);
    // ret is a clone of x-after-increment.
    assert_eq!(ret.clone_count(), 1);
    assert_eq!(x.clone_count(), 0);
}

// ── Sequence of postfix operations ───────────────────────────────────────────

#[test]
fn sequential_postfix_inc_captures_successive_values() {
    let mut x = Nc::new(0);
    let a = post_inc!(x);
    let b = post_inc!(x);
    let c = post_inc!(x);
    assert_eq!([a.val, b.val, c.val], [0, 1, 2]);
    assert_eq!(x.val, 3);
}

#[test]
fn mixed_prefix_postfix_sequence() {
    let mut x = Nc::new(0);
    let post = crement!(x++); // returns 0, x becomes 1
    let pre  = crement!(++x); // x becomes 2, returns 2
    assert_eq!(post.val, 0);
    assert_eq!(pre.val, 2);
    assert_eq!(x.val, 2);
}

// ── String as a real-world Clone-only type ────────────────────────────────────
//
// String doesn't implement AddAssign<i32>, so we demonstrate with a wrapper.

#[derive(Debug, Clone, PartialEq)]
struct Counter {
    label: String,
    count: i64,
}

impl AddAssign<i64> for Counter {
    fn add_assign(&mut self, rhs: i64) {
        self.count += rhs;
    }
}

impl SubAssign<i64> for Counter {
    fn sub_assign(&mut self, rhs: i64) {
        self.count -= rhs;
    }
}

#[test]
fn struct_with_string_field_postfix_inc() {
    let mut c = Counter { label: "hits".into(), count: 99 };
    let old = post_inc!(c);
    assert_eq!(old.count, 99);
    assert_eq!(old.label, "hits");
    assert_eq!(c.count, 100);
    // Verify the clone is deep: mutating label in c doesn't affect old.
    c.label = "changed".into();
    assert_eq!(old.label, "hits");
}

#[test]
fn struct_with_string_field_prefix_dec() {
    let mut c = Counter { label: "pages".into(), count: 10 };
    let new_val = pre_dec!(c);
    assert_eq!(new_val.count, 9);
    assert_eq!(c.count, 9);
}
