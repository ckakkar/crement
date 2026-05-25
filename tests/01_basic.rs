//! Basic round-trip tests for all four macro forms using i32 as a
//! representative Copy+Clone type.

use crement::{crement, post_dec, post_inc, pre_dec, pre_inc};

// ── crement!() unified macro ──────────────────────────────────────────────────

#[test]
fn crement_prefix_inc_returns_new_value() {
    let mut x: i32 = 5;
    let ret = crement!(++x);
    assert_eq!(ret, 6, "prefix ++ must return the new value");
    assert_eq!(x, 6, "prefix ++ must mutate in place");
}

#[test]
fn crement_postfix_inc_returns_old_value() {
    let mut x: i32 = 5;
    let ret = crement!(x++);
    assert_eq!(ret, 5, "postfix ++ must return the OLD value");
    assert_eq!(x, 6, "postfix ++ must mutate in place");
}

#[test]
fn crement_prefix_dec_returns_new_value() {
    let mut x: i32 = 5;
    let ret = crement!(--x);
    assert_eq!(ret, 4, "prefix -- must return the new value");
    assert_eq!(x, 4, "prefix -- must mutate in place");
}

#[test]
fn crement_postfix_dec_returns_old_value() {
    let mut x: i32 = 5;
    let ret = crement!(x--);
    assert_eq!(ret, 5, "postfix -- must return the OLD value");
    assert_eq!(x, 4, "postfix -- must mutate in place");
}

// ── pre_inc!() alias ──────────────────────────────────────────────────────────

#[test]
fn pre_inc_returns_new_value() {
    let mut x: i32 = 0;
    let ret = pre_inc!(x);
    assert_eq!(ret, 1);
    assert_eq!(x, 1);
}

#[test]
fn pre_inc_from_negative() {
    let mut x: i32 = -1;
    let ret = pre_inc!(x);
    assert_eq!(ret, 0);
    assert_eq!(x, 0);
}

// ── post_inc!() alias ─────────────────────────────────────────────────────────

#[test]
fn post_inc_returns_old_value() {
    let mut x: i32 = 0;
    let ret = post_inc!(x);
    assert_eq!(ret, 0);
    assert_eq!(x, 1);
}

#[test]
fn post_inc_across_zero() {
    let mut x: i32 = -1;
    let ret = post_inc!(x);
    assert_eq!(ret, -1);
    assert_eq!(x, 0);
}

// ── pre_dec!() alias ──────────────────────────────────────────────────────────

#[test]
fn pre_dec_returns_new_value() {
    let mut x: i32 = 5;
    let ret = pre_dec!(x);
    assert_eq!(ret, 4);
    assert_eq!(x, 4);
}

#[test]
fn pre_dec_into_negative() {
    let mut x: i32 = 0;
    let ret = pre_dec!(x);
    assert_eq!(ret, -1);
    assert_eq!(x, -1);
}

// ── post_dec!() alias ─────────────────────────────────────────────────────────

#[test]
fn post_dec_returns_old_value() {
    let mut x: i32 = 5;
    let ret = post_dec!(x);
    assert_eq!(ret, 5);
    assert_eq!(x, 4);
}

#[test]
fn post_dec_through_zero() {
    let mut x: i32 = 1;
    let ret = post_dec!(x);
    assert_eq!(ret, 1);
    assert_eq!(x, 0);
    let ret2 = post_dec!(x);
    assert_eq!(ret2, 0);
    assert_eq!(x, -1);
}

// ── return value is usable in expressions ────────────────────────────────────

#[test]
fn return_value_usable_in_arithmetic() {
    let mut a: i32 = 3;
    let mut b: i32 = 3;
    let sum = crement!(a++) + crement!(b++);
    assert_eq!(sum, 6); // 3 + 3
    assert_eq!(a, 4);
    assert_eq!(b, 4);
}

#[test]
fn prefix_return_usable_in_comparison() {
    let mut x: i32 = 4;
    assert!(crement!(++x) > 4);
    assert_eq!(x, 5);
}

#[test]
fn postfix_return_matches_original() {
    let mut x: i32 = 42;
    let snapshot = x;
    assert_eq!(post_inc!(x), snapshot);
    assert_eq!(x, snapshot + 1);
}

// ── macros produce expressions, not statements ───────────────────────────────

#[test]
fn can_discard_return_value() {
    let mut x: i32 = 0;
    let _ = crement!(x++); // discard return value — must not warn
    assert_eq!(x, 1);
    let _ = crement!(++x);
    assert_eq!(x, 2);
}

#[test]
fn can_use_in_let_binding() {
    let mut x: i32 = 7;
    let y = crement!(x++);
    let z = crement!(++x);
    assert_eq!(y, 7);
    assert_eq!(z, 9);
    assert_eq!(x, 9);
}
